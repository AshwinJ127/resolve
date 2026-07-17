import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, RotateCw, Activity, AlertTriangle } from "lucide-react";
import BranchList from "./components/BranchList";
import ActionBar from "./components/ActionBar";
import NewCommitModal from "./components/NewCommitModal";
import NewBranchModal from "./components/NewBranchModal";
import RemoteSelector from "./components/RemoteSelector";
import SwitchBranchModal from "./components/SwitchBranchModal";
import DirtyStateModal from "./components/DirtyStateModal";
import { RepoOverview, FileChange } from "./types/git";

type SyncState =
  | { kind: "idle" }
  | { kind: "syncing" }
  | { kind: "success"; ahead: number; behind: number }
  | { kind: "conflict"; files: string[] }
  | { kind: "error"; message: string };

function App() {
  const [status, setStatus] = useState("Loading...");
  const [repo, setRepo] = useState<RepoOverview | null>(null);
  const [commitOpen, setCommitOpen] = useState(false);
  const [branchOpen, setBranchOpen] = useState(false);
  const [switchBranchOpen, setSwitchBranchOpen] = useState(false);
  const [dirtyFiles, setDirtyFiles] = useState<FileChange[]>([]);
  const [dirtyModalOpen, setDirtyModalOpen] = useState(false);
  const [syncState, setSyncState] = useState<SyncState>({ kind: "idle" });
  const stashedRef = useRef(false);

  async function fetchStatus() {
    try {
      const result = await invoke("get_repo_overview");
      setRepo(result as RepoOverview);
      setStatus(JSON.stringify(result, null, 2));
    } catch (e) {
      setStatus("Error: " + e);
    }
  }

  async function runSync() {
    setSyncState({ kind: "syncing" });
    try {
      await invoke("smart_sync");
      const overview = await invoke("get_repo_overview") as RepoOverview;
      setRepo(overview);
      const current = overview.branches.find((b) => b.current);
      setSyncState({ kind: "success", ahead: current?.ahead ?? 0, behind: current?.behind ?? 0 });
      if (stashedRef.current) {
        await invoke("pop_stash");
        stashedRef.current = false;
        fetchStatus();
      }
    } catch (e) {
      const errMsg = String(e);
      if (errMsg.toLowerCase().includes("conflict")) {
        try {
          const conflicts = await invoke("get_conflict_files") as string[];
          setSyncState({ kind: "conflict", files: conflicts.length > 0 ? conflicts : ["Unknown — check your files"] });
        } catch {
          setSyncState({ kind: "conflict", files: ["Unknown — check your files"] });
        }
      } else {
        setSyncState({ kind: "error", message: errMsg });
        if (stashedRef.current) {
          await invoke("pop_stash").catch(() => {});
          stashedRef.current = false;
        }
      }
    }
  }

  async function handleSync() {
    setSyncState({ kind: "idle" });
    const pending = await invoke("get_pending_changes").catch(() => []) as FileChange[];
    if (pending.length > 0) {
      setDirtyFiles(pending);
      setDirtyModalOpen(true);
      return;
    }
    runSync();
  }

  async function handleDirtyCommit() {
    setDirtyModalOpen(false);
    setCommitOpen(true);
    // sync resumes via onSubmit callback on NewCommitModal
  }

  async function handleDirtyStash() {
    setDirtyModalOpen(false);
    try {
      await invoke("stash_changes");
      stashedRef.current = true;
      runSync();
    } catch (e) {
      setSyncState({ kind: "error", message: "Failed to stash: " + e });
    }
  }

  async function handleAbortMerge() {
    try {
      await invoke("abort_merge");
      if (stashedRef.current) {
        await invoke("pop_stash").catch(() => {});
        stashedRef.current = false;
      }
      setSyncState({ kind: "idle" });
      fetchStatus();
    } catch (e) {
      setSyncState({ kind: "error", message: "Failed to abort merge: " + e });
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

        <div className="flex items-center gap-4">
          <RemoteSelector />
          <button
            onClick={fetchStatus}
            className="group flex items-center justify-center rounded-lg border border-gray-800 bg-gray-900 p-2 text-gray-400 transition-colors hover:border-gray-700 hover:text-white"
            title="Refresh Status"
          >
            <RefreshCw size={18} className="transition-transform group-active:rotate-180" />
          </button>
          <button
            onClick={handleSync}
            disabled={syncState.kind === "syncing"}
            className="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-all hover:bg-blue-500 active:scale-95 disabled:opacity-60 disabled:cursor-not-allowed"
          >
            <RotateCw size={16} className={syncState.kind === "syncing" ? "animate-spin" : ""} />
            <span>{syncState.kind === "syncing" ? "Syncing..." : "Sync"}</span>
          </button>
        </div>
      </header>

      <main className="mx-auto max-w-7xl space-y-6 p-6">
        {status.startsWith("Error") && (
          <div className="rounded-lg border border-red-900/50 bg-red-950/30 p-4 text-sm text-red-400">
            <p className="font-mono">{status}</p>
          </div>
        )}

        <div className="space-y-6">
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
            onNewBranch={() => setBranchOpen(true)}
            onSwitchBranch={() => setSwitchBranchOpen(true)}
          />
        </div>
        
        {syncState.kind === "success" && (
          <div className="rounded-lg border border-green-900/50 bg-green-950/30 px-4 py-3 text-sm text-green-400">
            Up to date — {syncState.ahead} commit{syncState.ahead !== 1 ? "s" : ""} ahead, {syncState.behind} behind
          </div>
        )}

        {syncState.kind === "error" && (
          <div className="rounded-lg border border-red-900/50 bg-red-950/30 px-4 py-3 text-sm text-red-400 font-mono">
            {syncState.message}
          </div>
        )}

        {syncState.kind === "conflict" && (
          <div className="rounded-xl border border-orange-900/60 bg-orange-950/20 p-4 space-y-3">
            <div className="flex items-center gap-2 text-orange-400 font-semibold text-sm">
              <AlertTriangle size={16} />
              Merge Conflict
            </div>
            <p className="text-xs text-gray-400">
              Git couldn't automatically combine the changes. Open each file below and look for conflict markers (<code className="text-orange-300">&lt;&lt;&lt;&lt;&lt;&lt;&lt;</code>), then edit them to keep the right code.
            </p>
            <ul className="space-y-1">
              {syncState.files.map((f) => (
                <li key={f} className="text-xs font-mono text-orange-300">{f}</li>
              ))}
            </ul>
            <button
              onClick={handleAbortMerge}
              className="rounded-lg border border-orange-800 bg-orange-950/50 px-4 py-2 text-xs font-semibold text-orange-300 hover:bg-orange-900/40 transition-all"
            >
              Abort Merge — restore to before sync
            </button>
          </div>
        )}
      </main>

      <DirtyStateModal
        open={dirtyModalOpen}
        files={dirtyFiles}
        onCommit={handleDirtyCommit}
        onStash={handleDirtyStash}
        onCancel={() => setDirtyModalOpen(false)}
      />

      <NewCommitModal
        open={commitOpen}
        onClose={() => setCommitOpen(false)}
        onSubmit={() => {
          setCommitOpen(false);
          // if opened from sync flow, resume sync
          if (dirtyFiles.length > 0) {
            setDirtyFiles([]);
            runSync();
          } else {
            fetchStatus();
          }
        }}
      />

      <NewBranchModal
        open={branchOpen}
        onClose={() => setBranchOpen(false)}
        onSubmit={async (name) => {
          try {
            await invoke("create_branch", { name });
            setBranchOpen(false);
            fetchStatus();
          } catch (e) {
            alert("Branch creation failed: " + e);
          }
        }}
      />

      {repo && (
        <SwitchBranchModal
          open={switchBranchOpen}
          onClose={() => setSwitchBranchOpen(false)}
          onSubmit={() => {
            setSwitchBranchOpen(false);
            fetchStatus();
          }}
          branches={repo.branches}
        />
      )}
    </div>
  );
}

export default App;