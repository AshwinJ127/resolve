import { GitCommit, Archive, X } from "lucide-react";
import { FileChange } from "../types/git";

interface Props {
  open: boolean;
  files: FileChange[];
  onCommit: () => void;
  onStash: () => void;
  onCancel: () => void;
}

export default function DirtyStateModal({ open, files, onCommit, onStash, onCancel }: Props) {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-6">
      <div className="w-full max-w-md rounded-xl border border-gray-800 bg-gray-950 shadow-2xl">
        <div className="flex items-center justify-between border-b border-gray-800 px-6 py-4">
          <h3 className="text-base font-semibold text-gray-200">Unsaved Changes</h3>
          <button onClick={onCancel} className="text-gray-500 hover:text-white transition-colors">
            <X size={18} />
          </button>
        </div>

        <div className="px-6 py-4 space-y-3">
          <p className="text-sm text-gray-400">
            You have {files.length} unsaved change{files.length !== 1 ? "s" : ""}. To sync, you need to handle them first.
          </p>
          <ul className="max-h-36 overflow-y-auto space-y-1">
            {files.map((f) => (
              <li key={f.path} className="flex items-center gap-2 text-xs font-mono text-gray-500">
                <span className={
                  f.status === "??" ? "text-blue-400" :
                  f.status.startsWith("D") ? "text-red-400" : "text-yellow-400"
                }>
                  {f.status === "??" ? "new" : f.status.startsWith("D") ? "del" : "mod"}
                </span>
                <span className="truncate">{f.path}</span>
              </li>
            ))}
          </ul>
        </div>

        <div className="flex flex-col gap-2 border-t border-gray-800 px-6 py-4">
          <button
            onClick={onCommit}
            className="flex items-center gap-2 rounded-lg bg-green-700 px-4 py-2.5 text-sm font-semibold text-white hover:bg-green-600 transition-all"
          >
            <GitCommit size={16} />
            Commit changes, then sync
          </button>
          <button
            onClick={onStash}
            className="flex items-center gap-2 rounded-lg bg-gray-800 px-4 py-2.5 text-sm font-semibold text-gray-200 hover:bg-gray-700 transition-all"
          >
            <Archive size={16} />
            Stash changes and sync
          </button>
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm text-gray-500 hover:text-gray-300 transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
