import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BranchInfo, FileChange } from "../types/git";
import { GitMerge, Archive, AlertTriangle, Loader, GitCommit } from "lucide-react";

interface Props {
  open: boolean;
  onClose: () => void;
  onSubmit: () => void;
  branches: BranchInfo[];
}

type Action = "switch" | "stash" | "commit";

export default function SwitchBranchModal({ open, onClose, onSubmit, branches }: Props) {
  const [selectedBranch, setSelectedBranch] = useState<string>("");
  const [changes, setChanges] = useState<FileChange[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [action, setAction] = useState<Action | null>(null);
  const [isSwitching, setIsSwitching] = useState(false);

  useEffect(() => {
    if (open) {
      // Reset state on open
      setError(null);
      setSelectedBranch("");
      setIsSwitching(false);
      
      invoke<FileChange[]>("get_pending_changes")
        .then(res => {
          setChanges(res);
          // If there are no changes, default action is switch.
          // Otherwise, user must select an action.
          setAction(res.length > 0 ? null : "switch");
        })
        .catch(err => {
          setError(err.toString());
          setChanges([]);
        });
    }
  }, [open]);

  const handleSwitch = async () => {
    if (!selectedBranch || isSwitching) return;

    if (changes.length > 0 && !action) {
      setError("Please select an action for your uncommitted changes.");
      return;
    }

    setIsSwitching(true);
    setError(null);

    try {
      if (changes.length > 0) {
        if (action === "stash") {
          await invoke("stash_changes");
        } else if (action === "commit") {
          setError("Committing from here is not implemented yet.");
          setIsSwitching(false);
          return;
        }
      }

      await invoke("switch_branch", { name: selectedBranch });

      if (changes.length > 0 && action === "stash") {
        try {
          await invoke("pop_stash");
        } catch (popError) {
          // The switch succeeded, but the pop failed.
          // We should still submit and let the user know.
          alert("Switched branch, but could not apply stashed changes. Please run 'git stash pop' manually to resolve conflicts.");
        }
      }

      onSubmit();
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setIsSwitching(false);
    }
  };

  const isSwitchDisabled = !selectedBranch || isSwitching || (changes.length > 0 && !action);

  if (!open) {
    return null;
  }

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 w-full max-w-md shadow-2xl">
        <div className="flex items-center gap-3 mb-4">
            <GitMerge className="text-blue-400" />
            <h2 className="text-lg font-bold">Switch Branch</h2>
        </div>

        {error && 
            <div className="text-red-400 bg-red-900/50 border border-red-800 rounded-md p-3 mb-4 text-sm">
                {error}
            </div>
        }
        
        {changes.length > 0 && (
          <div className="mb-4 p-4 bg-yellow-900/40 border border-yellow-700/60 text-yellow-200 rounded-lg">
            <div className="flex items-center gap-2 mb-3">
                <AlertTriangle size={18} />
                <p className="font-bold text-sm">You have uncommitted changes</p>
            </div>
            <div className="space-y-2">
              <label className={`flex items-center p-3 rounded-md transition-colors ${action === 'stash' ? 'bg-blue-600/30 border-blue-500' : 'bg-gray-700/50 border-gray-600'} border cursor-pointer`}>
                <input type="radio" name="action" value="stash" checked={action === "stash"} onChange={() => setAction("stash")} className="hidden" />
                <Archive size={16} className="mr-3 text-blue-300" />
                <span className="text-sm">Stash changes and switch</span>
              </label>
              <label className="flex items-center p-3 rounded-md bg-gray-700/30 border-gray-700 border cursor-not-allowed opacity-50">
                <input type="radio" name="action" value="commit" disabled className="hidden" />
                <GitCommit size={16} className="mr-3" />
                <span className="text-sm">Commit changes (not implemented)</span>
              </label>
            </div>
          </div>
        )}

        <select 
          value={selectedBranch}
          onChange={(e) => setSelectedBranch(e.target.value)}
          className="w-full bg-gray-900 border border-gray-600 rounded-md p-2 mb-4 text-white focus:ring-2 focus:ring-blue-500 focus:outline-none"
        >
          <option value="" disabled>Select a branch to switch to</option>
          {branches.filter(b => !b.current).map(branch => (
            <option key={branch.name} value={branch.name}>{branch.name}</option>
          ))}
        </select>

        <div className="flex justify-end gap-3 mt-6">
          <button onClick={onClose} className="bg-gray-600 hover:bg-gray-500 py-2 px-4 rounded-lg text-sm font-semibold transition-colors">Cancel</button>
          <button 
            onClick={handleSwitch} 
            disabled={isSwitchDisabled} 
            className="bg-blue-600 hover:bg-blue-500 py-2 px-4 rounded-lg text-sm font-semibold transition-all flex items-center gap-2 disabled:bg-gray-500 disabled:cursor-not-allowed"
          >
            {isSwitching && <Loader size={16} className="animate-spin" />}
            {isSwitching ? "Switching..." : (action === "stash" ? "Stash & Switch" : "Switch Branch")}
          </button>
        </div>
      </div>
    </div>
  );
}
