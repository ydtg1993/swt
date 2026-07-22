import { useI18n } from "@/hooks/useI18n";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";

export default function AboutPage() {
  const { t } = useI18n();
  return (
    <div className="space-y-4 max-w-2xl">
      <h2 className="text-xl font-extrabold uppercase">{t("nav.about")}</h2>

      <Card className="border-2 border-black dark:border-white">
        <CardHeader className="text-center pb-2">
          <p className="text-5xl mb-3">🎙️</p>
          <CardTitle className="text-3xl font-black tracking-tight">{t("app.name")}</CardTitle>
          <Badge className="mx-auto mt-2 border-2 border-black dark:border-white bg-primary text-primary-foreground font-extrabold px-3 py-1 text-sm">v0.1.0</Badge>
        </CardHeader>
        <CardContent className="space-y-4 text-center">
          <p className="text-sm font-bold text-muted-foreground max-w-md mx-auto">
            离线语音转文字桌面应用 — 基于音频 AI 推理引擎，无需联网，保护你的隐私数据。
          </p>

          <Separator className="border-black dark:border-white" />

          <div className="grid grid-cols-2 gap-3 text-left">
            <div className="border-2 border-black dark:border-white p-3">
              <p className="text-[10px] font-bold uppercase text-muted-foreground mb-1">框架</p>
              <p className="text-sm font-bold">Tauri v2 + React 19</p>
            </div>
            <div className="border-2 border-black dark:border-white p-3">
              <p className="text-[10px] font-bold uppercase text-muted-foreground mb-1">推理引擎</p>
              <p className="text-sm font-bold">whisper.cpp / audio.cpp</p>
            </div>
            <div className="border-2 border-black dark:border-white p-3">
              <p className="text-[10px] font-bold uppercase text-muted-foreground mb-1">语言</p>
              <p className="text-sm font-bold">Rust / TypeScript</p>
            </div>
            <div className="border-2 border-black dark:border-white p-3">
              <p className="text-[10px] font-bold uppercase text-muted-foreground mb-1">许可证</p>
              <p className="text-sm font-bold">MIT</p>
            </div>
          </div>

          <div className="flex flex-wrap justify-center gap-2 text-[10px] font-bold">
            <Badge variant="outline" className="border-2 border-black dark:border-white">Tailwind CSS</Badge>
            <Badge variant="outline" className="border-2 border-black dark:border-white">Shadcn/UI</Badge>
            <Badge variant="outline" className="border-2 border-black dark:border-white">whisper.cpp</Badge>
            <Badge variant="outline" className="border-2 border-black dark:border-white">symphonia</Badge>
            <Badge variant="outline" className="border-2 border-black dark:border-white">rubato</Badge>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
