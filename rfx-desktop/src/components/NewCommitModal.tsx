import { useState } from "react";

interface Props {
  open: boolean;
  onClose: () => void;
  onSubmit: (message: string) => void;
}

export default function NewCommitModal({ open, onClose, onSubmit }: Props) {
  const [msg, setMsg] = useState("");

  if (!open) return null;

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center">
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-4 w-80">
        <h3 className="text-sm mb-2 text-gray-300">New Commit</h3>

        <textarea
          value={msg}
          onChange={e => setMsg(e.target.value)}
          className="w-full h-20 bg-black border border-gray-700 rounded p-2 text-sm font-mono"
          placeholder="Describe your change…"
        />

        <div className="flex justify-end gap-2 mt-3">
          <button onClick={onClose} className="text-gray-400 text-sm">
            Cancel
          </button>
          <button
            onClick={() => {
              onSubmit(msg);
              setMsg("");
            }}
            className="bg-green-600 px-3 py-1 rounded text-sm"
          >
            Commit
          </button>
        </div>
      </div>
    </div>
  );
}
