import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const overview = {
  branches: [
    { name: "main", current: true, ahead: 0, behind: 0, author: "Test", date: "2026-01-01", message: "init" },
  ],
};

describe("App sync flow", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === "get_repo_overview") {
        return Promise.resolve(overview);
      } else if (command === "get_remotes") {
        return Promise.resolve([]);
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
  });

  it("syncs immediately when there are no pending changes", async () => {
    render(<App />);
    await waitFor(() => expect(screen.getByText("main")).toBeInTheDocument());

    mockedInvoke.mockImplementation((command: string) => {
      if (command === "get_pending_changes") {
        return Promise.resolve([]);
      } else if (command === "smart_sync") {
        return Promise.resolve("Sync complete!");
      } else if (command === "get_repo_overview") {
        return Promise.resolve(overview);
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });

    fireEvent.click(screen.getByRole("button", { name: /^Sync$/ }));

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("smart_sync"));
    expect(screen.queryByText("Unsaved Changes")).not.toBeInTheDocument();
  });

  it("shows the dirty-state modal instead of syncing immediately when changes are pending", async () => {
    render(<App />);
    await waitFor(() => expect(screen.getByText("main")).toBeInTheDocument());

    mockedInvoke.mockImplementation((command: string) => {
      if (command === "get_pending_changes") {
        return Promise.resolve([{ path: "a.txt", status: "M" }]);
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });

    fireEvent.click(screen.getByRole("button", { name: /^Sync$/ }));

    await waitFor(() => expect(screen.getByText("Unsaved Changes")).toBeInTheDocument());
    expect(mockedInvoke).not.toHaveBeenCalledWith("smart_sync");
  });

  it("stashing from the dirty modal stashes, syncs, then pops the stash", async () => {
    render(<App />);
    await waitFor(() => expect(screen.getByText("main")).toBeInTheDocument());

    mockedInvoke.mockImplementation((command: string) => {
      if (command === "get_pending_changes") {
        return Promise.resolve([{ path: "a.txt", status: "M" }]);
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });

    fireEvent.click(screen.getByRole("button", { name: /^Sync$/ }));
    await waitFor(() => expect(screen.getByText("Unsaved Changes")).toBeInTheDocument());

    mockedInvoke.mockImplementation((command: string) => {
      if (command === "stash_changes") {
        return Promise.resolve(undefined);
      } else if (command === "smart_sync") {
        return Promise.resolve("Sync complete!");
      } else if (command === "get_repo_overview") {
        return Promise.resolve(overview);
      } else if (command === "pop_stash") {
        return Promise.resolve(undefined);
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });

    fireEvent.click(screen.getByText("Stash changes and sync"));

    // Wait for all three commands to be invoked
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("pop_stash"));

    // Verify the commands were called in the correct order
    const commandOrder = mockedInvoke.mock.calls.map((call) => call[0]);
    const stashIndex = commandOrder.indexOf("stash_changes");
    const syncIndex = commandOrder.indexOf("smart_sync");
    const popIndex = commandOrder.indexOf("pop_stash");

    // Verify all commands were called
    expect(stashIndex).toBeGreaterThanOrEqual(0);
    expect(syncIndex).toBeGreaterThanOrEqual(0);
    expect(popIndex).toBeGreaterThanOrEqual(0);

    // Verify they were called in the correct order
    expect(syncIndex).toBeGreaterThan(stashIndex);
    expect(popIndex).toBeGreaterThan(syncIndex);
  });
});
