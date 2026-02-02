import { BranchInfo } from "../types/git";

interface Props {
  branches: BranchInfo[];
}

export default function BranchList({ branches }: Props) {
  return (
    <div className="bg-black/40 rounded-lg border border-gray-800 p-3 mb-4">
      <h2 className="text-xs text-gray-400 mb-2 uppercase tracking-wide">
        Branches
      </h2>

      <div className="space-y-2">
        {branches.map(branch => (
          <div
            key={branch.name}
            className={`flex items-center justify-between rounded px-2 py-1 text-sm font-mono
              ${branch.current ? "bg-blue-900/40 text-blue-300" : "bg-gray-900 text-gray-300"}
            `}
          >
            <span>{branch.name}</span>

            <span className="text-xs text-gray-400">
              {branch.ahead > 0 && `↑${branch.ahead} `}
              {branch.behind > 0 && `↓${branch.behind}`}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
