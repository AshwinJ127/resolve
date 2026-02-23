import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Globe } from "lucide-react";

interface Remote {
  name: string;
  url: string;
}

export default function RemoteSelector() {
  const [remotes, setRemotes] = useState<Remote[]>([]);
  const [selectedRemote, setSelectedRemote] = useState<string>("");
  const [isLoading, setIsLoading] = useState(true);

  async function fetchRemotes() {
    try {
      const result = await invoke("get_remotes");
      const fetchedRemotes = result as Remote[];
      setRemotes(fetchedRemotes);
      // Heuristic to find the current remote, find a better way later
      const upstream = fetchedRemotes.find(r => r.name === "origin") || fetchedRemotes[0];
      if (upstream) {
        setSelectedRemote(upstream.name);
      }
    } catch (e) {
      console.error("Failed to fetch remotes", e);
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    fetchRemotes();
  }, []);

  const handleRemoteChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const remoteName = e.target.value;
    setSelectedRemote(remoteName);
    try {
      await invoke("switch_remote", { remote: remoteName });
      // Show success toast
    } catch (error) {
      console.error("Failed to switch remote", error);
      // Show error toast
    }
  };

  if (isLoading) {
    return <div>Loading remotes...</div>;
  }

  return (
    <div className="flex items-center gap-2">
      <Globe size={16} className="text-gray-500" />
      <select
        value={selectedRemote}
        onChange={handleRemoteChange}
        className="bg-gray-800 border border-gray-700 rounded-md px-2 py-1 text-xs text-white"
      >
        {remotes.map((remote) => (
          <option key={remote.name} value={remote.name}>
            {remote.name}
          </option>
        ))}
      </select>
    </div>
  );
}
