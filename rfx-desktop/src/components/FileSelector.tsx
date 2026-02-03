import { useState, useMemo } from "react";
import { Folder, FileCode, Check, ChevronRight, ChevronDown } from "lucide-react";
import { FileChange } from "../types/git";

interface Props {
  files: FileChange[];
  selected: Set<string>;
  onToggle: (path: string) => void;
  onToggleAll: () => void;
}

// Helper to build tree from paths
const buildTree = (files: FileChange[]) => {
  const root: any = {};
  files.forEach((file) => {
    const parts = file.path.split("/");
    let current = root;
    parts.forEach((part, i) => {
      if (!current[part]) {
        current[part] = i === parts.length - 1 ? { _file: file } : {};
      }
      current = current[part];
    });
  });
  return root;
};

export default function FileSelector({ files, selected, onToggle, onToggleAll }: Props) {
  const isTreeMode = files.length > 10;
  const allSelected = files.length > 0 && selected.size === files.length;

  return (
    <div className="flex flex-col h-full border border-gray-800 rounded-lg bg-black/40 overflow-hidden">
      {/* Header with Select All */}
      <div className="flex items-center justify-between px-3 py-2 bg-gray-900/50 border-b border-gray-800">
        <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
          {files.length} Changed Files
        </span>
        <button
          onClick={onToggleAll}
          className="text-[10px] uppercase font-bold text-blue-400 hover:text-blue-300"
        >
          {allSelected ? "Deselect All" : "Select All"}
        </button>
      </div>

      {/* Scrollable List Area */}
      <div className="flex-1 overflow-y-auto p-2">
        {isTreeMode ? (
          <FileTree node={buildTree(files)} onToggle={onToggle} selected={selected} />
        ) : (
          <FlatList files={files} selected={selected} onToggle={onToggle} />
        )}
      </div>
    </div>
  );
}

function FlatList({ files, selected, onToggle }: any) {
  return (
    <div className="space-y-1">
      {files.map((file: FileChange) => (
        <FileRow
          key={file.path}
          file={file}
          checked={selected.has(file.path)}
          onToggle={() => onToggle(file.path)}
        />
      ))}
    </div>
  );
}

// Recursive Tree Component
function FileTree({ node, prefix = "", onToggle, selected }: any) {
  const entries = Object.entries(node);

  return (
    <div className="pl-3 border-l border-gray-800/50 ml-1">
      {entries.map(([key, value]: any) => {
        if (value._file) {
          // It's a file
          return (
            <FileRow
              key={value._file.path}
              file={value._file}
              checked={selected.has(value._file.path)}
              onToggle={() => onToggle(value._file.path)}
            />
          );
        } else {
          // It's a folder
          return (
            <FolderRow key={key} name={key}>
               <FileTree node={value} onToggle={onToggle} selected={selected} />
            </FolderRow>
          );
        }
      })}
    </div>
  );
}

function FolderRow({ name, children }: any) {
  const [open, setOpen] = useState(true);
  return (
    <div>
      <div
        className="flex items-center gap-2 py-1 cursor-pointer text-gray-400 hover:text-white"
        onClick={() => setOpen(!open)}
      >
        {open ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
        <Folder size={14} className="text-blue-500/60" />
        <span className="text-sm font-medium">{name}</span>
      </div>
      {open && children}
    </div>
  );
}

function FileRow({ file, checked, onToggle }: any) {
  // Color code status
  const statusColor =
    file.status === "M" ? "text-yellow-400" :
    file.status === "??" ? "text-green-400" :
    file.status === "D" ? "text-red-400" : "text-gray-400";

  return (
    <div
      onClick={onToggle}
      className={`flex items-center gap-3 px-2 py-1.5 rounded cursor-pointer group select-none transition-colors
        ${checked ? "bg-blue-900/20" : "hover:bg-white/5"}
      `}
    >
      <div className={`w-4 h-4 rounded border flex items-center justify-center transition-all
        ${checked ? "bg-blue-600 border-blue-500" : "border-gray-600 bg-transparent group-hover:border-gray-500"}
      `}>
        {checked && <Check size={12} className="text-white" />}
      </div>
      
      <span className={`text-[10px] font-mono w-6 text-center shrink-0 ${statusColor}`}>
        [{file.status === "??" ? "NEW" : file.status}]
      </span>

      <div className="flex items-center gap-2 overflow-hidden">
        <FileCode size={14} className="text-gray-500 shrink-0" />
        <span className={`text-sm truncate font-mono ${checked ? "text-gray-200" : "text-gray-400"}`}>
          {file.path.split("/").pop()} 
          {/* Note: In flat mode we might want full path, but strictly filename is cleaner if context is clear */}
        </span>
      </div>
    </div>
  );
}