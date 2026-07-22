import { useState, useEffect } from "react";
import { useI18n } from "@/hooks/useI18n";
import { invokeCmd, getConfig } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export default function ApiServerPage() {
  const { t } = useI18n();
  const [running, setRunning] = useState(false);
  const [port, setPort] = useState(8000);
  const [cppPort, setCppPort] = useState(8080);
  const [apiKey, setApiKey] = useState("");
  const [msg, setMsg] = useState("");

  useEffect(() => {
    getConfig().then((c) => setPort(c.api_server.port)).catch(console.error);
  }, []);

  const toggle = async () => {
    if (running) {
      setRunning(false);
      setMsg("已停止（请重启应用以完全释放端口）");
    } else {
      try {
        await invokeCmd("start_api_server", { port, cppPort, apiKey });
        setRunning(true);
        setMsg("API 服务器已启动 ✓");
      } catch (e) { setMsg(`启动失败: ${e}`); }
    }
  };

  return (
    <div className="space-y-4 max-w-2xl">
      <h2 className="text-xl font-extrabold uppercase">{t("nav.apiServer")}</h2>

      <Card className="border-2 border-black dark:border-white">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <CardTitle className="text-base font-extrabold">🔌 OpenAI 兼容 API</CardTitle>
            <div className="flex items-center gap-2">
              <span className={`h-3 w-3 rounded-full border-2 border-black dark:border-white ${running ? "bg-green-400 animate-pulse" : "bg-red-400"}`} />
              <span className="text-sm font-bold">{running ? `运行中 :${port}` : "已停止"}</span>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap gap-3 items-end">
            <div>
              <label className="text-xs font-bold uppercase block mb-1">API 端口</label>
              <Input type="number" value={port} onChange={(e) => setPort(Number(e.target.value))}
                className="w-24 text-sm font-bold" />
            </div>
            <div>
              <label className="text-xs font-bold uppercase block mb-1">推理端口</label>
              <Input type="number" value={cppPort} onChange={(e) => setCppPort(Number(e.target.value))}
                className="w-24 text-sm font-bold" />
            </div>
            <div>
              <label className="text-xs font-bold uppercase block mb-1">API Key (可选)</label>
              <Input type="password" value={apiKey} onChange={(e) => setApiKey(e.target.value)}
                className="w-40 text-sm font-bold" placeholder="留空则不验证" />
            </div>
            <Button onClick={toggle} variant={running ? "destructive" : "default"}
              className="font-bold border-2 border-black dark:border-white shadow-[3px_3px_0px_#000] dark:shadow-[3px_3px_0px_#fff]">
              {running ? `⏹ ${t("common.stop")}` : `▶ ${t("common.start")}`}
            </Button>
          </div>
          {msg && <p className={`text-sm font-bold px-3 py-1 border-2 border-black dark:border-white ${running ? "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300" : "bg-muted text-muted-foreground"}`}>{msg}</p>}

          <div className="border-2 border-black dark:border-white bg-muted/30 p-4 space-y-1 text-sm font-mono">
            <p className="font-bold text-xs uppercase mb-2">可用端点：</p>
            <p><span className="font-bold text-green-600">GET</span>  /health</p>
            <p><span className="font-bold text-green-600">GET</span>  /v1/models</p>
            <p><span className="font-bold text-blue-600">POST</span> /v1/audio/transcriptions</p>
            <p><span className="font-bold text-blue-600">POST</span> /v1/audio/speech</p>
          </div>

          <div className="border-2 border-black dark:border-white p-3 bg-muted/20">
            <code className="block text-xs break-all font-mono">
              <span className="text-muted-foreground">$</span> curl -X POST http://localhost:{port}/v1/audio/transcriptions \<br/>
              <span className="ml-4">-F "</span>file=@audio.wav<span>" -F "</span>model=qwen3<span>"</span>
            </code>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
