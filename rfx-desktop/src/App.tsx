import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, RotateCw, Terminal, Activity } from "lucide-react";
import BranchList from "./components/BranchList";
import ActionBar from "./components/ActionBar";
import NewCommitModal from "./components/NewCommitModal";
import { RepoOverview } from "./types/git";

function App() {
  const [status, setStatus] = useState("Loading...");
  const [log, setLog] = useState("");
  const [repo, setRepo] = useState<RepoOverview | null>(null);
  const [commitOpen, setCommitOpen] = useState(false);

  async function fetchStatus() {
    try {
      const result = await invoke("get_repo_overview");
      setRepo(result as RepoOverview);
      setStatus(JSON.stringify(result, null, 2));
    } catch (e) {
      setStatus("Error: " + e);
    }
  }

  async function handleSync() {
    setLog("Syncing...");
    try {
      const result = await invoke("smart_sync");
      setLog(result as string);
      fetchStatus();
    } catch (e) {
      setLog("Error: " + e);
    }
  }

  useEffect(() => {
    fetchStatus();
  }, []);

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100 font-sans selection:bg-blue-500/30">
      <header className="sticky top-0 z-10 flex h-16 items-center justify-between border-b border-gray-800 bg-gray-950/80 px-6 backdrop-blur-md">
        <div className="flex items-center gap-3">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-600/20 text-blue-400">
            <Activity size={18} />
          </div>
          <div>
            <h1 className="text-sm font-bold tracking-wide text-gray-100">RFX DASHBOARD</h1>
            <div className="flex items-center gap-2">
              <span className="h-1.5 w-1.5 rounded-full bg-green-500"></span>
              <span className="text-xs font-medium text-gray-500">v0.1.0</span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={fetchStatus}
            className="group flex items-center justify-center rounded-lg border border-gray-800 bg-gray-900 p-2 text-gray-400 transition-colors hover:border-gray-700 hover:text-white"
            title="Refresh Status"
          >
            <RefreshCw size={18} className="transition-transform group-active:rotate-180" />
          </button>
          <button
            onClick={handleSync}
            className="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-all hover:bg-blue-500 active:scale-95"
          >
            <RotateCw size={16} />
            <span>Sync</span>
          </button>
        </div>
      </header>

      <main className="mx-auto max-w-5xl space-y-6 p-6">
        {status.startsWith("Error") && (
          <div className="rounded-lg border border-red-900/50 bg-red-950/30 p-4 text-sm text-red-400">
            <p className="font-mono">{status}</p>
          </div>
        )}

        <div className="grid gap-6 lg:grid-cols-3">
          <div className="lg:col-span-2 space-y-6">
            <div className="rounded-xl border border-gray-800 bg-gray-900/50 p-1">
              {repo ? (
                <BranchList branches={repo.branches} />
              ) : (
                <div className="flex h-32 items-center justify-center text-sm text-gray-500">
                  No repository data loaded
                </div>
              )}
            </div>

            <ActionBar
              onCommit={() => setCommitOpen(true)}
              onNewBranch={() => invoke("create_branch")}
              onSync={handleSync}
            />
          </div>

          <div className="lg:col-span-1">
            <div className="h-full rounded-xl border border-gray-800 bg-black/40 p-4">
              <div className="mb-3 flex items-center gap-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">
                <Terminal size={14} />
                <span>System Logs</span>
              </div>
              <div className="font-mono text-xs text-gray-400 break-all leading-relaxed">
                {log || <span className="text-gray-700">Waiting for actions...</span>}
              </div>
            </div>
          </div>
        </div>
      </main>

      <NewCommitModal
        open={commitOpen}
        onClose={() => setCommitOpen(false)}
        onSubmit={async (msg) => {
          await invoke("create_commit", { message: msg });
          setCommitOpen(false);
          fetchStatus();
        }}
      />
    </div>
  );
}

export default App;