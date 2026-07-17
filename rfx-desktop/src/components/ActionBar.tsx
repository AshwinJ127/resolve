interface Props {
  onCommit: () => void;
  onNewBranch: () => void;
  onSwitchBranch: () => void;
}

export default function ActionBar({ onCommit, onNewBranch, onSwitchBranch }: Props) {
  return (
    <div className="grid grid-cols-3 gap-2 mt-4">
      <button
        onClick={onCommit}
        className="bg-green-600 hover:bg-green-500 py-2 rounded-lg text-sm font-medium"
      >
        Commit
      </button>

      <button
        onClick={onNewBranch}
        className="bg-purple-600 hover:bg-purple-500 py-2 rounded-lg text-sm font-medium"
      >
        New Branch
      </button>
      
      <button
        onClick={onSwitchBranch}
        className="bg-blue-600 hover:bg-blue-500 py-2 rounded-lg text-sm font-medium"
      >
        Switch Branch
      </button>
    </div>
  );
}
