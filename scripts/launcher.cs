using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Net;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading;
using System.Windows.Forms;

namespace DPLSFastLauncher
{
    static class Program
    {
        private const int BACKEND_PORT = 6800;
        private const int COMPANION_PORT = 6805;
        private static string appDir = AppDomain.CurrentDomain.BaseDirectory;
        private static HttpListener listener;
        private static Thread listenerThread;
        private static bool isStopping = false;

        [STAThread]
        static void Main()
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);

            // 1. Ensure backend daemon is running on port 6800
            EnsureBackendRunning();

            // 2. Check if companion server on port 6805 is already alive
            bool companionRunning = IsPortAlive(COMPANION_PORT);
            if (!companionRunning)
            {
                StartCompanionServer();
            }

            // 3. Open UI window in standalone App Mode
            string targetUrl = "http://127.0.0.1:" + COMPANION_PORT + "/";
            OpenAppWindow(targetUrl);

            // 4. If we started the companion server, enter the Tray application message loop
            if (listener != null && listener.IsListening)
            {
                Application.Run(new TrayApplicationContext());
            }
        }

        private static void EnsureBackendRunning()
        {
            if (IsPortAlive(BACKEND_PORT)) return;

            string dplsGui = Path.Combine(appDir, "dpls-gui.exe");
            if (!File.Exists(dplsGui)) dplsGui = Path.Combine(appDir, "dpls-desktop.exe");
            if (!File.Exists(dplsGui)) dplsGui = Path.Combine(appDir, "dpls.exe");

            if (File.Exists(dplsGui))
            {
                ProcessStartInfo psi = new ProcessStartInfo
                {
                    FileName = dplsGui,
                    WorkingDirectory = appDir,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };
                if (Path.GetFileName(dplsGui).Equals("dpls.exe", StringComparison.OrdinalIgnoreCase))
                {
                    psi.Arguments = "server --port " + BACKEND_PORT;
                }
                try { Process.Start(psi); } catch { }

                for (int i = 0; i < 60; i++)
                {
                    Thread.Sleep(100);
                    if (IsPortAlive(BACKEND_PORT)) break;
                }
            }
        }

        private static bool IsPortAlive(int port)
        {
            try
            {
                HttpWebRequest req = (HttpWebRequest)WebRequest.Create("http://127.0.0.1:" + port + "/api/tasks");
                req.Timeout = 400;
                req.Method = "GET";
                using (HttpWebResponse resp = (HttpWebResponse)req.GetResponse())
                {
                    return resp.StatusCode == HttpStatusCode.OK;
                }
            }
            catch
            {
                return false;
            }
        }

        private static void StartCompanionServer()
        {
            try
            {
                listener = new HttpListener();
                listener.Prefixes.Add("http://127.0.0.1:" + COMPANION_PORT + "/");
                listener.Start();

                listenerThread = new Thread(() =>
                {
                    while (!isStopping && listener.IsListening)
                    {
                        try
                        {
                            HttpListenerContext ctx = listener.GetContext();
                            ThreadPool.QueueUserWorkItem((state) => HandleRequest((HttpListenerContext)state), ctx);
                        }
                        catch
                        {
                            if (isStopping) break;
                        }
                    }
                });
                listenerThread.IsBackground = true;
                listenerThread.Start();
            }
            catch { }
        }

        public static void StopCompanionServer()
        {
            isStopping = true;
            try
            {
                if (listener != null)
                {
                    listener.Stop();
                    listener.Close();
                }
            }
            catch { }
        }

        private static void HandleRequest(HttpListenerContext ctx)
        {
            try
            {
                string rawUrl = ctx.Request.RawUrl ?? "/";
                string path = ctx.Request.Url.AbsolutePath;

                // Add CORS and Private Network Access headers
                ctx.Response.AddHeader("Access-Control-Allow-Origin", "*");
                ctx.Response.AddHeader("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
                ctx.Response.AddHeader("Access-Control-Allow-Headers", "*");
                ctx.Response.AddHeader("Access-Control-Allow-Private-Network", "true");

                if (ctx.Request.HttpMethod == "OPTIONS")
                {
                    ctx.Response.StatusCode = 204;
                    ctx.Response.Close();
                    return;
                }

                // 1. Root page: Serve web/index.html
                if (path == "/" || path == "/index.html")
                {
                    ServeIndexHtml(ctx);
                    return;
                }

                // 2. Folder Browser Dialog: POST /api/dialog/pick-dir
                if (path.Equals("/api/dialog/pick-dir", StringComparison.OrdinalIgnoreCase))
                {
                    string selected = PickFolder();
                    string json = selected != null
                        ? "{\"path\":" + EscapeJson(selected) + ",\"canceled\":false}"
                        : "{\"path\":null,\"canceled\":true}";
                    SendJson(ctx, json);
                    return;
                }

                // 3. Open File: POST /api/open/{id}
                if (path.StartsWith("/api/open/", StringComparison.OrdinalIgnoreCase))
                {
                    string id = path.Substring(10).Trim('/');
                    HandleOpenFile(ctx, id);
                    return;
                }

                // 4. Open Folder & Select File: POST /api/open-dir/{id}
                if (path.StartsWith("/api/open-dir/", StringComparison.OrdinalIgnoreCase))
                {
                    string id = path.Substring(14).Trim('/');
                    HandleOpenDir(ctx, id);
                    return;
                }

                // 5. System Clipboard: GET /api/system/clipboard
                if (path.Equals("/api/system/clipboard", StringComparison.OrdinalIgnoreCase))
                {
                    string clipText = GetClipboardText();
                    SendJson(ctx, "{\"text\":" + EscapeJson(clipText) + "}");
                    return;
                }

                // 6. Proxy everything else to 127.0.0.1:6800
                ProxyRequest(ctx);
            }
            catch
            {
                try
                {
                    ctx.Response.StatusCode = 500;
                    ctx.Response.Close();
                }
                catch { }
            }
        }

        private static void ServeIndexHtml(HttpListenerContext ctx)
        {
            string htmlPath = Path.Combine(appDir, @"web\index.html");
            if (!File.Exists(htmlPath)) htmlPath = Path.Combine(appDir, "index.html");
            if (!File.Exists(htmlPath))
            {
                string parentWeb = Path.Combine(appDir, @"..\web\index.html");
                if (File.Exists(parentWeb)) htmlPath = parentWeb;
            }

            if (File.Exists(htmlPath))
            {
                byte[] bytes = File.ReadAllBytes(htmlPath);
                ctx.Response.ContentType = "text/html; charset=utf-8";
                ctx.Response.ContentLength64 = bytes.Length;
                if (ctx.Request.HttpMethod != "HEAD")
                {
                    ctx.Response.OutputStream.Write(bytes, 0, bytes.Length);
                }
                ctx.Response.Close();
            }
            else
            {
                ProxyRequest(ctx);
            }
        }

        private static string PickFolder()
        {
            string selected = null;
            Thread t = new Thread(() =>
            {
                using (FolderBrowserDialog fbd = new FolderBrowserDialog())
                {
                    fbd.Description = "選擇下載儲存路徑";
                    fbd.ShowNewFolderButton = true;
                    using (Form form = new Form { TopMost = true, TopLevel = true, StartPosition = FormStartPosition.CenterScreen })
                    {
                        if (fbd.ShowDialog(form) == DialogResult.OK)
                        {
                            selected = fbd.SelectedPath;
                        }
                    }
                }
            });
            t.SetApartmentState(ApartmentState.STA);
            t.Start();
            t.Join();
            return selected;
        }

        private static string GetQueryParam(HttpListenerContext ctx, string name)
        {
            try
            {
                if (ctx.Request.Url != null && !string.IsNullOrEmpty(ctx.Request.Url.Query))
                {
                    var qs = System.Web.HttpUtility.ParseQueryString(ctx.Request.Url.Query, Encoding.UTF8);
                    string val = qs[name];
                    if (!string.IsNullOrEmpty(val)) return val;
                }
            }
            catch { }
            try
            {
                return ctx.Request.QueryString[name];
            }
            catch { }
            return null;
        }

        private static string ResolveTargetFilePath(HttpListenerContext ctx, string id)
        {
            string filePath = GetQueryParam(ctx, "path");
            if (string.IsNullOrEmpty(filePath) && ctx.Request.HasEntityBody)
            {
                try
                {
                    using (StreamReader sr = new StreamReader(ctx.Request.InputStream, Encoding.UTF8))
                    {
                        string body = sr.ReadToEnd();
                        Match m = Regex.Match(body, "\"path\"\\s*:\\s*\"(.*?)(?<!\\\\)\"");
                        if (m.Success)
                        {
                            filePath = m.Groups[1].Value.Replace("\\\\", "\\").Replace("\\\"", "\"").Replace("\\/", "/");
                        }
                    }
                }
                catch { }
            }

            if (string.IsNullOrEmpty(filePath) && !string.IsNullOrEmpty(id) && id != "_")
            {
                filePath = GetTaskOutputPath(id);
            }

            if (!string.IsNullOrEmpty(filePath))
            {
                filePath = filePath.Trim('"', '\'', ' ').Replace('/', '\\');
            }
            return filePath;
        }

        private static void HandleOpenFile(HttpListenerContext ctx, string id)
        {
            string filePath = ResolveTargetFilePath(ctx, id);
            if (!string.IsNullOrEmpty(filePath) && File.Exists(filePath))
            {
                try
                {
                    ProcessStartInfo psi = new ProcessStartInfo(filePath) { UseShellExecute = true };
                    Process.Start(psi);
                    SendJson(ctx, "{\"ok\":true,\"path\":" + EscapeJson(filePath) + "}");
                    return;
                }
                catch (Exception ex)
                {
                    SendJson(ctx, "{\"ok\":false,\"error\":" + EscapeJson(ex.Message) + "}");
                    return;
                }
            }
            ctx.Response.StatusCode = 404;
            SendJson(ctx, "{\"ok\":false,\"error\":\"File not found\",\"path\":" + EscapeJson(filePath ?? "") + "}");
        }

        private static void HandleOpenDir(HttpListenerContext ctx, string id)
        {
            string filePath = ResolveTargetFilePath(ctx, id);
            if (!string.IsNullOrEmpty(filePath))
            {
                try
                {
                    if (File.Exists(filePath))
                    {
                        ProcessStartInfo psi = new ProcessStartInfo("explorer.exe", "/select,\"" + filePath + "\"")
                        {
                            UseShellExecute = true
                        };
                        Process.Start(psi);
                        SendJson(ctx, "{\"ok\":true,\"target\":\"file\",\"path\":" + EscapeJson(filePath) + "}");
                        return;
                    }
                    else
                    {
                        string dir = Directory.Exists(filePath) ? filePath : Path.GetDirectoryName(filePath);
                        if (!string.IsNullOrEmpty(dir) && Directory.Exists(dir))
                        {
                            ProcessStartInfo psi = new ProcessStartInfo("explorer.exe", "\"" + dir + "\"")
                            {
                                UseShellExecute = true
                            };
                            Process.Start(psi);
                            SendJson(ctx, "{\"ok\":true,\"target\":\"dir\",\"path\":" + EscapeJson(dir) + "}");
                            return;
                        }
                    }
                }
                catch (Exception ex)
                {
                    SendJson(ctx, "{\"ok\":false,\"error\":" + EscapeJson(ex.Message) + "}");
                    return;
                }
            }
            ctx.Response.StatusCode = 404;
            SendJson(ctx, "{\"ok\":false,\"error\":\"Directory or file not found\",\"path\":" + EscapeJson(filePath ?? "") + "}");
        }

        private static string GetTaskOutputPath(string id)
        {
            if (string.IsNullOrEmpty(id)) return null;

            // 1. Try single task endpoint: /api/task/{id}
            try
            {
                HttpWebRequest req = (HttpWebRequest)WebRequest.Create("http://127.0.0.1:" + BACKEND_PORT + "/api/task/" + Uri.EscapeDataString(id));
                req.Timeout = 1500;
                req.Method = "GET";
                using (HttpWebResponse resp = (HttpWebResponse)req.GetResponse())
                using (StreamReader reader = new StreamReader(resp.GetResponseStream(), Encoding.UTF8))
                {
                    string json = reader.ReadToEnd();
                    Match m = Regex.Match(json, "\"output_path\"\\s*:\\s*\"(.*?)(?<!\\\\)\"");
                    if (m.Success)
                    {
                        return m.Groups[1].Value.Replace("\\\\", "\\").Replace("\\\"", "\"").Replace("\\/", "/");
                    }
                }
            }
            catch { }

            // 2. Fallback to /api/tasks list
            try
            {
                HttpWebRequest req = (HttpWebRequest)WebRequest.Create("http://127.0.0.1:" + BACKEND_PORT + "/api/tasks");
                req.Timeout = 2000;
                req.Method = "GET";
                using (HttpWebResponse resp = (HttpWebResponse)req.GetResponse())
                using (StreamReader reader = new StreamReader(resp.GetResponseStream(), Encoding.UTF8))
                {
                    string json = reader.ReadToEnd();
                    string pattern = "\"id\"\\s*:\\s*\"" + Regex.Escape(id) + "\"[^}]+?\"output_path\"\\s*:\\s*\"(.*?)(?<!\\\\)\"";
                    Match m = Regex.Match(json, pattern, RegexOptions.Singleline);
                    if (m.Success)
                    {
                        return m.Groups[1].Value.Replace("\\\\", "\\").Replace("\\\"", "\"").Replace("\\/", "/");
                    }
                }
            }
            catch { }
            return null;
        }

        private static string GetClipboardText()
        {
            string text = "";
            Thread t = new Thread(() =>
            {
                try
                {
                    if (Clipboard.ContainsText())
                    {
                        text = Clipboard.GetText();
                    }
                }
                catch { }
            });
            t.SetApartmentState(ApartmentState.STA);
            t.Start();
            t.Join();
            return text ?? "";
        }

        private static void ProxyRequest(HttpListenerContext context)
        {
            try
            {
                string targetUrl = "http://127.0.0.1:" + BACKEND_PORT + context.Request.RawUrl;
                HttpWebRequest proxyReq = (HttpWebRequest)WebRequest.Create(targetUrl);
                proxyReq.Method = context.Request.HttpMethod;
                proxyReq.Timeout = 10000;
                if (!string.IsNullOrEmpty(context.Request.ContentType))
                {
                    proxyReq.ContentType = context.Request.ContentType;
                }
                if (!string.IsNullOrEmpty(context.Request.UserAgent))
                {
                    proxyReq.UserAgent = context.Request.UserAgent;
                }

                if (context.Request.HasEntityBody)
                {
                    using (Stream reqStream = proxyReq.GetRequestStream())
                    {
                        context.Request.InputStream.CopyTo(reqStream);
                    }
                }

                using (HttpWebResponse proxyResp = (HttpWebResponse)proxyReq.GetResponse())
                {
                    context.Response.StatusCode = (int)proxyResp.StatusCode;
                    context.Response.ContentType = proxyResp.ContentType;
                    using (Stream respStream = proxyResp.GetResponseStream())
                    {
                        respStream.CopyTo(context.Response.OutputStream);
                    }
                }
            }
            catch (WebException we)
            {
                HttpWebResponse wr = we.Response as HttpWebResponse;
                if (wr != null)
                {
                    context.Response.StatusCode = (int)wr.StatusCode;
                }
                else
                {
                    context.Response.StatusCode = 502;
                }
            }
            catch
            {
                context.Response.StatusCode = 500;
            }
            finally
            {
                try { context.Response.Close(); } catch { }
            }
        }

        private static void SendJson(HttpListenerContext ctx, string json)
        {
            byte[] bytes = Encoding.UTF8.GetBytes(json);
            ctx.Response.ContentType = "application/json; charset=utf-8";
            ctx.Response.ContentLength64 = bytes.Length;
            if (ctx.Request.HttpMethod != "HEAD")
            {
                ctx.Response.OutputStream.Write(bytes, 0, bytes.Length);
            }
            ctx.Response.Close();
        }

        private static string EscapeJson(string s)
        {
            if (s == null) return "null";
            StringBuilder sb = new StringBuilder("\"");
            foreach (char c in s)
            {
                if (c == '"') sb.Append("\\\"");
                else if (c == '\\') sb.Append("\\\\");
                else if (c == '\b') sb.Append("\\b");
                else if (c == '\f') sb.Append("\\f");
                else if (c == '\n') sb.Append("\\n");
                else if (c == '\r') sb.Append("\\r");
                else if (c == '\t') sb.Append("\\t");
                else sb.Append(c);
            }
            sb.Append('"');
            return sb.ToString();
        }

        public static void OpenAppWindow(string url)
        {
            // 1. Try Microsoft Edge App Mode
            string edgePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
                @"Microsoft\Edge\Application\msedge.exe");
            if (!File.Exists(edgePath))
            {
                edgePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles),
                    @"Microsoft\Edge\Application\msedge.exe");
            }

            if (File.Exists(edgePath))
            {
                try
                {
                    Process.Start(new ProcessStartInfo
                    {
                        FileName = edgePath,
                        Arguments = "--app=\"" + url + "\" --window-size=1260,840",
                        UseShellExecute = true
                    });
                    return;
                }
                catch { }
            }

            // 2. Try Google Chrome App Mode
            string chromePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles),
                @"Google\Chrome\Application\chrome.exe");
            if (!File.Exists(chromePath))
            {
                chromePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
                    @"Google\Chrome\Application\chrome.exe");
            }
            if (!File.Exists(chromePath))
            {
                chromePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    @"Google\Chrome\Application\chrome.exe");
            }

            if (File.Exists(chromePath))
            {
                try
                {
                    Process.Start(new ProcessStartInfo
                    {
                        FileName = chromePath,
                        Arguments = "--app=\"" + url + "\" --window-size=1260,840",
                        UseShellExecute = true
                    });
                    return;
                }
                catch { }
            }

            // 3. Fallback to default browser
            try
            {
                Process.Start(new ProcessStartInfo(url) { UseShellExecute = true });
            }
            catch { }
        }
    }

    class TrayApplicationContext : ApplicationContext
    {
        private NotifyIcon trayIcon;

        public TrayApplicationContext()
        {
            ContextMenu menu = new ContextMenu();
            menu.MenuItems.Add(new MenuItem("開啟 DPLS-Fast", (s, e) => Program.OpenAppWindow("http://127.0.0.1:6805/")));
            menu.MenuItems.Add(new MenuItem("-"));
            menu.MenuItems.Add(new MenuItem("離開 DPLS-Fast", (s, e) =>
            {
                Program.StopCompanionServer();
                trayIcon.Visible = false;
                Application.Exit();
            }));

            trayIcon = new NotifyIcon
            {
                Text = "DPLS-Fast 極速多線程下載器",
                ContextMenu = menu,
                Visible = true
            };

            string iconPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "icon.ico");
            if (File.Exists(iconPath))
            {
                try { trayIcon.Icon = new Icon(iconPath); } catch { }
            }
            if (trayIcon.Icon == null)
            {
                trayIcon.Icon = SystemIcons.Application;
            }

            trayIcon.DoubleClick += (s, e) => Program.OpenAppWindow("http://127.0.0.1:6805/");
        }
    }
}
