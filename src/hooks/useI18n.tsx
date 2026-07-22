import { createContext, useContext, useState, useCallback, type ReactNode } from "react";
import zh from "@/i18n/zh";
import en from "@/i18n/en";

type Lang = "zh" | "en";
type TranslationKeys = keyof typeof zh;

const translations = { zh, en } as const;

interface I18nCtx {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: TranslationKeys) => string;
}

const Ctx = createContext<I18nCtx>({
  lang: "zh",
  setLang: () => {},
  t: (k) => String(k),
});

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLang] = useState<Lang>(() => {
    const saved = localStorage.getItem("swt2-lang");
    return saved === "en" ? "en" : "zh";
  });

  const t = useCallback(
    (key: TranslationKeys): string => {
      return translations[lang][key] ?? translations.zh[key] ?? String(key);
    },
    [lang],
  );

  const changeLang = (l: Lang) => {
    setLang(l);
    localStorage.setItem("swt2-lang", l);
  };

  return <Ctx.Provider value={{ lang, setLang: changeLang, t }}>{children}</Ctx.Provider>;
}

export function useI18n() {
  return useContext(Ctx);
}
