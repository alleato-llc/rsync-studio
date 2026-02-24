import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Sidebar, type NavPage } from "@/components/sidebar";
import { JobsPage } from "@/pages/jobs-page";
import { HistoryPage } from "@/pages/history-page";
import { StatisticsPage } from "@/pages/statistics-page";
import { ToolsPage } from "@/pages/tools-page";
import { SettingsPage } from "@/pages/settings-page";
import { AboutPage } from "@/pages/about-page";
import { useTheme } from "@/hooks/use-theme";

function App() {
  const [currentPage, setCurrentPage] = useState<NavPage>("jobs");
  useTheme();

  useEffect(() => {
    const unlisten = listen("navigate-to-about", () => {
      setCurrentPage("about");
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <main className="flex-1 overflow-y-auto p-8">
        {currentPage === "jobs" && <JobsPage />}
        {currentPage === "history" && <HistoryPage />}
        {currentPage === "statistics" && <StatisticsPage />}
        {currentPage === "tools" && <ToolsPage />}
        {currentPage === "settings" && <SettingsPage />}
        {currentPage === "about" && <AboutPage />}
      </main>
    </div>
  );
}

export default App;
