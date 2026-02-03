interface Props {
  onCommit: () => void;
  onNewBranch: () => void;
}

export default function ActionBar({ onCommit, onNewBranch}: Props) {
  return (
    <div className="grid grid-cols-2 gap-2 mt-4">
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
        Branch
      </button>
    </div>
  );
}
