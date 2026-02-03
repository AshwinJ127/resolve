import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X, GitCommit, Loader2 } from "lucide-react";
import FileSelector from "./FileSelector";
import { FileChange } from "../types/git";

interface Props {
  open: boolean;
  onClose: () => void;
  onSubmit: () => void; 
}

export default function NewCommitModal({ open, onClose, onSubmit }: Props) {
  const [msg, setMsg] = useState("");
  const [files, setFiles] = useState<FileChange[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [committing, setCommitting] = useState(false);

  // Load changes when modal opens
  useEffect(() => {
    if (open) {
      setLoading(true);
      invoke("get_pending_changes")
        .then((data) => {
          const fileList = data as FileChange[];
          setFiles(fileList);
          // Default: Select All
          setSelected(new Set(fileList.map((f) => f.path)));
          setLoading(false);
        })
        .catch((err) => {
          console.error(err);
          setLoading(false);
        });
    } else {
        setMsg("");
    }
  }, [open]);

  const toggleFile = (path: string) => {
    const next = new Set(selected);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    setSelected(next);
  };

  const toggleAll = () => {
    if (selected.size === files.length) {
      setSelected(new Set());
    } else {
      setSelected(new Set(files.map(f => f.path)));
    }
  };

  const handleCommit = async () => {
    console.log("1. Button clicked");
    
    if (!msg.trim()) {
      alert("Message is empty");
      return;
    }
    if (selected.size === 0) {
      alert("No files selected");
      return;
    }

    setCommitting(true);
    try {
      console.log("2. Invoking commit_selection with:", { 
        message: msg, 
        files: Array.from(selected) 
      });

      const response = await invoke("commit_selection", { 
        message: msg, 
        files: Array.from(selected) 
      });

      console.log("3. Rust Response:", response);

      // 4. Update UI
      onSubmit(); 
      onClose();

    } catch (e) {
      console.error("COMMIT FAILED:", e);
      alert("Error: " + e);
    } finally {
      setCommitting(false);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-6">
      <div className="w-full max-w-4xl h-[600px] rounded-xl border border-gray-800 bg-gray-950 flex flex-col shadow-2xl overflow-hidden">
        
        {/* Header */}
        <div className="h-16 border-b border-gray-800 flex items-center justify-between px-6 bg-gray-900/30">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-green-500/10 rounded-lg text-green-400">
              <GitCommit size={20} />
            </div>
            <h3 className="text-lg font-semibold text-gray-200">Stage & Commit</h3>
          </div>
          <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
            <X size={20} />
          </button>
        </div>

        {/* Content - Split View */}
        <div className="flex-1 flex overflow-hidden">
            
          {/* LEFT: File Selection */}
          <div className="w-1/2 p-4 border-r border-gray-800 flex flex-col gap-2">
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
              Unsaved Changes
            </h4>
            
            {loading ? (
              <div className="flex-1 flex items-center justify-center text-gray-600">
                <Loader2 className="animate-spin mr-2" /> Loading changes...
              </div>
            ) : files.length === 0 ? (
              <div className="flex-1 flex items-center justify-center text-gray-600 italic">
                No changes found.
              </div>
            ) : (
              <FileSelector 
                files={files} 
                selected={selected} 
                onToggle={toggleFile} 
                onToggleAll={toggleAll}
              />
            )}
          </div>

          {/* RIGHT: Message Input */}
          <div className="w-1/2 p-4 flex flex-col bg-gray-900/10">
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-4">
              Commit Message
            </h4>
            
            <textarea
              autoFocus
              value={msg}
              onChange={(e) => setMsg(e.target.value)}
              className="flex-1 w-full rounded-lg border border-gray-800 bg-black/50 p-4 text-sm text-white placeholder-gray-600 focus:border-green-500 focus:outline-none focus:ring-1 focus:ring-green-500 transition-all font-mono resize-none mb-4"
              placeholder="feat: implemented file tree selection..."
            />

            <div className="flex items-center justify-between pt-2 border-t border-gray-800/50">
               <span className="text-xs text-gray-500">
                 {selected.size} file(s) selected
               </span>
               <button
                  onClick={handleCommit}
                  disabled={!msg.trim() || selected.size === 0 || committing}
                  className="rounded-lg bg-green-600 px-6 py-2.5 text-sm font-bold text-white hover:bg-green-500 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
                >
                  {committing && <Loader2 size={16} className="animate-spin" />}
                  Commit Changes
                </button>
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}