import { BranchInfo } from "../types/git";
import { GitBranch, GitCommit, User, Calendar, ArrowUp, ArrowDown, Globe } from "lucide-react";

interface Props {
  branches: BranchInfo[];
}

export default function BranchList({ branches }: Props) {

  return (
    <div className="bg-black/40 rounded-xl border border-gray-800 overflow-hidden flex flex-col h-full">
      {/* Header */}
      <div className="px-4 py-3 border-b border-gray-800 flex justify-between items-center bg-gray-900/30">
        <div className="flex items-center gap-2">
          <GitBranch size={16} className="text-gray-500" />
          <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
            Local Branches
          </h2>
        </div>
        <span className="text-xs text-gray-600 font-mono px-2 py-0.5 rounded bg-gray-900 border border-gray-800">
          {branches.length}
        </span>
      </div>

      {/* Table Container */}
      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm border-collapse">
          <thead className="bg-gray-900/50 text-xs uppercase text-gray-500 font-medium sticky top-0 z-10 backdrop-blur-sm">
            <tr>
              <th className="px-4 py-3 w-[25%]">Branch / Upstream</th>
              <th className="px-4 py-3 w-[10%] text-center">Sync</th>
              <th className="px-4 py-3 w-[35%]">Last Commit</th>
              <th className="px-4 py-3 w-[15%]">Author</th>
              <th className="px-4 py-3 w-[15%] text-right">Updated</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800/50">
            {branches.map((branch) => (
              <tr
                key={branch.name}
                className={`group transition-colors hover:bg-white/5 ${
                  branch.current 
                    ? "bg-blue-900/10 hover:bg-blue-900/20" 
                    : ""
                }`}
              >
                {/* 1. Branch Name & Upstream */}
                <td className="px-4 py-3 align-top">
                  <div className="flex flex-col gap-1">
                    <div className="flex items-center gap-2">
                      <span className={`font-mono font-bold tracking-tight ${
                        branch.current ? "text-blue-400" : "text-gray-200"
                      }`}>
                        {branch.name}
                      </span>
                      {branch.current && (
                        <span className="px-1.5 py-0.5 rounded-[4px] text-[10px] font-bold bg-blue-500/20 text-blue-300 border border-blue-500/30 shadow-[0_0_10px_rgba(59,130,246,0.2)]">
                          HEAD
                        </span>
                      )}
                    </div>
                    {branch.upstream && (
                      <div className="flex items-center gap-1 text-[11px] text-gray-500 font-mono group-hover:text-gray-400 transition-colors">
                        <Globe size={10} />
                        <span>{branch.upstream}</span>
                      </div>
                    )}
                  </div>
                </td>

                {/* 2. Sync Status (Ahead/Behind) */}
                <td className="px-4 py-3 align-middle">
                  <div className="flex justify-center gap-3">
                    {branch.ahead > 0 ? (
                      <div className="flex items-center gap-0.5 text-green-400 font-mono text-xs" title={`${branch.ahead} commits ahead`}>
                        <ArrowUp size={12} strokeWidth={3} />
                        <span>{branch.ahead}</span>
                      </div>
                    ) : (
                      <div className="w-4" /> 
                    )}
                    
                    {branch.behind > 0 ? (
                      <div className="flex items-center gap-0.5 text-red-400 font-mono text-xs" title={`${branch.behind} commits behind`}>
                        <ArrowDown size={12} strokeWidth={3} />
                        <span>{branch.behind}</span>
                      </div>
                    ) : (
                      <div className="w-4" />
                    )}

                    {branch.ahead === 0 && branch.behind === 0 && branch.upstream && (
                      <span className="text-gray-600 text-[10px] font-mono">Synced</span>
                    )}
                  </div>
                </td>

                {/* 3. Commit Message */}
                <td className="px-4 py-3 align-top">
                  <div className="flex gap-2 text-gray-400 max-w-sm">
                    <GitCommit size={14} className="mt-1 shrink-0 opacity-40 group-hover:opacity-100 transition-opacity" />
                    <span className="truncate leading-relaxed text-gray-300 group-hover:text-white transition-colors" title={branch.message}>
                      {branch.message}
                    </span>
                  </div>
                </td>

                {/* 4. Author */}
                <td className="px-4 py-3 align-top">
                  <div className="flex items-center gap-2 text-gray-400 text-xs">
                    <div className="w-5 h-5 rounded-full bg-gray-800 flex items-center justify-center text-gray-500 shrink-0">
                        <User size={12} />
                    </div>
                    <span className="truncate max-w-[100px]">{branch.author}</span>
                  </div>
                </td>

                {/* 5. Date */}
                <td className="px-4 py-3 align-top text-right text-gray-500 font-mono text-xs whitespace-n-wrap">
                  <div className="flex items-center justify-end gap-1.5 h-full">
                    <span>{branch.date}</span>
                    <Calendar size={12} className="opacity-40" />
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}