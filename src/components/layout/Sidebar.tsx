import { useState } from "react";
import { NavLink } from "react-router-dom";
import { useI18n } from "@/hooks/useI18n";
import { useTheme } from "@/hooks/useTheme";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

const NAV_ITEMS = [
  { path: "/", key: "nav.transcribe" as const, icon: "📝" },
  { path: "/realtime", key: "nav.realtime" as const, icon: "🎙️" },
  { path: "/conversation", key: "nav.conversation" as const, icon: "💬" },
  { path: "/tts", key: "nav.tts" as const, icon: "🔊" },
  { path: "/models", key: "nav.models" as const, icon: "🧩" },
  { path: "/history", key: "nav.history" as const, icon: "📋" },
  { path: "/api-server", key: "nav.apiServer" as const, icon: "🔌" },
  { path: "/settings", key: "nav.settings" as const, icon: "⚙️" },
  { path: "/about", key: "nav.about" as const, icon: "ℹ️" },
];

export default function Sidebar() {
  const { t, lang, setLang } = useI18n();
  const { theme, toggle } = useTheme();
  const [collapsed, setCollapsed] = useState(false);

  return (
    <aside
      className={`flex flex-col border-r-2 border-black dark:border-white bg-card transition-all duration-200 ${
        collapsed ? "w-16" : "w-56"}`}
      aria-label="Sidebar navigation"
    >
      <div className="flex h-14 items-center justify-between border-b-2 border-black dark:border-white px-3">
        {!collapsed && <span className="text-sm font-extrabold uppercase tracking-tight">{t("app.name")}</span>}
        <Button size="sm" variant="outline" onClick={() => setCollapsed(!collapsed)}
          className="font-bold border-2 border-black dark:border-white shrink-0 text-xs h-7 w-7 p-0"
          aria-expanded={!collapsed}>
          {collapsed ? "▸" : "◂"}
        </Button>
      </div>

      <nav className="flex-1 overflow-y-auto py-2" aria-label="Main navigation">
        {NAV_ITEMS.map((item) => (
          <NavLink key={item.path} to={item.path} end={item.path === "/"}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2.5 mx-2 my-0.5 text-sm font-bold border-2 transition-all ${
                isActive
                  ? "bg-yellow-300 dark:bg-yellow-600 border-black dark:border-white shadow-[3px_3px_0px_#000] dark:shadow-[3px_3px_0px_#fff]"
                  : "border-transparent hover:border-black dark:hover:border-white hover:bg-muted"}`}>
            <span className="text-lg shrink-0">{item.icon}</span>
            {!collapsed && <span className="truncate uppercase text-xs tracking-wide">{t(item.key)}</span>}
          </NavLink>
        ))}
      </nav>

      <Separator className="border-black dark:border-white" />
      <div className="p-2 flex flex-col gap-1">
        <Button variant="ghost" size="sm" onClick={toggle}
          className="justify-start gap-2 font-bold text-xs border-2 border-transparent hover:border-black dark:hover:border-white">
          <span className="text-lg shrink-0">{theme === "light" ? "🌙" : "☀️"}</span>
          {!collapsed && (theme === "light" ? "DARK" : "LIGHT")}
        </Button>
        <Button variant="ghost" size="sm" onClick={() => setLang(lang === "zh" ? "en" : "zh")}
          className="justify-start gap-2 font-bold text-xs border-2 border-transparent hover:border-black dark:hover:border-white">
          <span className="text-lg shrink-0">🌐</span>
          {!collapsed && (lang === "zh" ? "EN" : "中")}
        </Button>
      </div>
    </aside>
  );
}
