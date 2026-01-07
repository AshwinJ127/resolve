import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [status, setStatus] = useState("Loading status...");

  async function fetchStatus() {
    try {
      const result = await invoke("get_git_status");
      setStatus(result as string);
    } catch (e) {
      setStatus("Error connecting to backend: " + e);
    }
  }

  // Fetch status automatically when the app starts
  useEffect(() => {
    fetchStatus();
  }, []);

  return (
    <div className="container">
      <h1>rfx Dashboard</h1>
      
      <div className="status-box">
        {/* We use <pre> to preserve the line breaks from your Rust format! */}
        <pre>{status}</pre>
      </div>

      <button onClick={fetchStatus}>
        Refresh Status
      </button>
    </div>
  );
}

export default App;