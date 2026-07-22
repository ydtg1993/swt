import { HashRouter, Routes, Route } from "react-router-dom";
import { ThemeProvider } from "@/hooks/useTheme";
import { I18nProvider } from "@/hooks/useI18n";
import Layout from "@/components/layout/Layout";
import TranscribePage from "@/pages/TranscribePage";
import RealTimePage from "@/pages/RealTimePage";
import ConversationPage from "@/pages/ConversationPage";
import TtsPage from "@/pages/TtsPage";
import ModelManagerPage from "@/pages/ModelManagerPage";
import HistoryPage from "@/pages/HistoryPage";
import ApiServerPage from "@/pages/ApiServerPage";
import SettingsPage from "@/pages/SettingsPage";
import AboutPage from "@/pages/AboutPage";

function App() {
  return (
    <ThemeProvider>
      <I18nProvider>
        <HashRouter>
          <Layout>
            <Routes>
              <Route path="/" element={<TranscribePage />} />
              <Route path="/realtime" element={<RealTimePage />} />
              <Route path="/conversation" element={<ConversationPage />} />
              <Route path="/tts" element={<TtsPage />} />
              <Route path="/models" element={<ModelManagerPage />} />
              <Route path="/history" element={<HistoryPage />} />
              <Route path="/api-server" element={<ApiServerPage />} />
              <Route path="/settings" element={<SettingsPage />} />
              <Route path="/about" element={<AboutPage />} />
            </Routes>
          </Layout>
        </HashRouter>
      </I18nProvider>
    </ThemeProvider>
  );
}

export default App;
