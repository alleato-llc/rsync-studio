import { useState } from "react";
import { Sidebar, type NavPage } from "@/components/sidebar";
import { JobsPage } from "@/pages/jobs-page";
import { HistoryPage } from "@/pages/history-page";
import { StatisticsPage } from "@/pages/statistics-page";
import { ToolsPage } from "@/pages/tools-page";
import { SettingsPage } from "@/pages/settings-page";

function App() {
  const [currentPage, setCurrentPage] = useState<NavPage>("jobs");

  return (
    <div className="flex min-h-screen">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <main className="flex-1 p-8">
        {currentPage === "jobs" && <JobsPage />}
        {currentPage === "history" && <HistoryPage />}
        {currentPage === "statistics" && <StatisticsPage />}
        {currentPage === "tools" && <ToolsPage />}
        {currentPage === "settings" && <SettingsPage />}
      </main>
    </div>
  );
}

export default App;
