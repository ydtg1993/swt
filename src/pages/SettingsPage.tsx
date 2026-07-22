import { useEffect, useState } from "react";
import { useI18n } from "@/hooks/useI18n";
import { getConfig, setConfig, resetConfig, type AppConfig } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export default function SettingsPage() {
  const { t } = useI18n();
  const [cfg, setCfg] = useState<AppConfig | null>(null);
  const [msg, setMsg] = useState("");

  useEffect(() => {
    getConfig().then(setCfg).catch(console.error);
  }, []);

  const update = async (key: string, value: unknown) => {
    try {
      await setConfig(key, value);
      setCfg((prev) => prev ? { ...prev } : prev);
      setMsg("已保存 ✓");
      setTimeout(() => setMsg(""), 2000);
    } catch (e) { setMsg(`保存失败: ${e}`); }
  };

  const reset = async () => {
    try {
      const fresh = await resetConfig();
      setCfg(fresh);
      setMsg("已恢复默认 ✓");
    } catch (e) { setMsg(`重置失败: ${e}`); }
  };

  if (!cfg) return <div className="p-8 text-muted-foreground font-bold">{t("common.loading")}</div>;

  const Field = ({ label, children }: { label: string; children: React.ReactNode }) => (
    <div className="flex items-center justify-between py-2.5 border-b-2 border-black/20 dark:border-white/20 last:border-0">
      <span className="text-sm font-bold">{label}</span>
      <div className="w-64">{children}</div>
    </div>
  );

  return (
    <div className="space-y-4 max-w-2xl">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-extrabold uppercase">{t("nav.settings")}</h2>
        {msg && <span className="text-sm font-bold text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30 px-3 py-1 border-2 border-green-500">{msg}</span>}
      </div>

      {/* ASR */}
      <Card className="border-2 border-black dark:border-white">
        <CardHeader className="pb-2"><CardTitle className="text-base font-extrabold">🎙️ ASR 推理</CardTitle></CardHeader>
        <CardContent className="space-y-1">
          <Field label="默认模型">
            <Input value={cfg.asr.default_model} onChange={(e) => update("asr.default_model", e.target.value)}
              className="text-sm font-bold" />
          </Field>
          <Field label="设备">
            <select value={cfg.asr.device} onChange={(e) => update("asr.device", e.target.value)}
              className="w-full border-2 border-black dark:border-white bg-background px-3 py-2 text-sm font-bold rounded-lg">
              <option value="auto">auto</option>
              <option value="cpu">CPU</option>
              <option value="cuda:0">CUDA GPU</option>
            </select>
          </Field>
          <Field label="默认语言">
            <select value={cfg.asr.language} onChange={(e) => update("asr.language", e.target.value)}
              className="w-full border-2 border-black dark:border-white bg-background px-3 py-2 text-sm font-bold rounded-lg">
              <option value="auto">auto</option>
              <option value="zh">中文</option>
              <option value="en">English</option>
              <option value="ja">日本語</option>
              <option value="ko">한국어</option>
            </select>
          </Field>
        </CardContent>
      </Card>

      {/* TTS */}
      <Card className="border-2 border-black dark:border-white">
        <CardHeader className="pb-2"><CardTitle className="text-base font-extrabold">🔊 TTS</CardTitle></CardHeader>
        <CardContent className="space-y-1">
          <Field label="默认模型">
            <Input value={cfg.tts.default_model} onChange={(e) => update("tts.default_model", e.target.value)}
              className="text-sm font-bold" />
          </Field>
          <Field label="音色">
            <Input value={cfg.tts.voice} onChange={(e) => update("tts.voice", e.target.value)}
              className="text-sm font-bold" />
          </Field>
          <Field label="语速">
            <div className="flex items-center gap-2">
              <input type="range" min="0.5" max="2.0" step="0.1" value={cfg.tts.speed}
                onChange={(e) => update("tts.speed", parseFloat(e.target.value))}
                className="flex-1 accent-primary" />
              <span className="text-xs font-bold w-10 text-right">{cfg.tts.speed}x</span>
            </div>
          </Field>
        </CardContent>
      </Card>

      {/* UI */}
      <Card className="border-2 border-black dark:border-white">
        <CardHeader className="pb-2"><CardTitle className="text-base font-extrabold">🖥️ 界面</CardTitle></CardHeader>
        <CardContent className="space-y-1">
          <Field label="主题">
            <select value={cfg.ui.theme} onChange={(e) => update("ui.theme", e.target.value)}
              className="w-full border-2 border-black dark:border-white bg-background px-3 py-2 text-sm font-bold rounded-lg">
              <option value="light">{t("theme.light")}</option>
              <option value="dark">{t("theme.dark")}</option>
            </select>
          </Field>
          <Field label="语言">
            <select value={cfg.ui.language} onChange={(e) => update("ui.language", e.target.value)}
              className="w-full border-2 border-black dark:border-white bg-background px-3 py-2 text-sm font-bold rounded-lg">
              <option value="zh">中文</option>
              <option value="en">English</option>
            </select>
          </Field>
        </CardContent>
      </Card>

      {/* LLM */}
      <Card className="border-2 border-black dark:border-white">
        <CardHeader className="pb-2"><CardTitle className="text-base font-extrabold">🤖 LLM / AI 修正</CardTitle></CardHeader>
        <CardContent className="space-y-1">
          <Field label="启用">
            <input type="checkbox" checked={cfg.llm.enabled}
              onChange={(e) => update("llm.enabled", e.target.checked)}
              className="w-5 h-5 accent-primary border-2 border-black dark:border-white" />
          </Field>
          <Field label="API URL">
            <Input value={cfg.llm.api_url} onChange={(e) => update("llm.api_url", e.target.value)}
              className="text-sm font-bold" />
          </Field>
          <Field label="API Key">
            <Input type="password" value={cfg.llm.api_key} onChange={(e) => update("llm.api_key", e.target.value)}
              className="text-sm font-bold" />
          </Field>
          <Field label="模型">
            <Input value={cfg.llm.model} onChange={(e) => update("llm.model", e.target.value)}
              className="text-sm font-bold" />
          </Field>
        </CardContent>
      </Card>

      {/* Export */}
      <Card className="border-2 border-black dark:border-white">
        <CardHeader className="pb-2"><CardTitle className="text-base font-extrabold">📁 导出</CardTitle></CardHeader>
        <CardContent>
          <Field label="默认目录">
            <Input value={cfg.export.output_dir} onChange={(e) => update("export.output_dir", e.target.value)}
              className="text-sm font-bold" />
          </Field>
        </CardContent>
      </Card>

      <Button variant="destructive" onClick={reset}
        className="font-bold border-2 border-black dark:border-white shadow-[3px_3px_0px_#000] dark:shadow-[3px_3px_0px_#fff]">
        🗑 恢复默认设置
      </Button>
    </div>
  );
}
