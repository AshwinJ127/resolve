import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import RemoteSelector from "./RemoteSelector";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

describe("RemoteSelector", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("shows a loading state before remotes arrive", () => {
    mockedInvoke.mockReturnValueOnce(new Promise(() => {})); // never resolves
    render(<RemoteSelector />);
    expect(screen.getByText("Loading remotes...")).toBeInTheDocument();
  });

  it("lists only fetch-direction remotes, defaulting to origin", async () => {
    mockedInvoke.mockResolvedValueOnce([
      { name: "origin", url: "https://github.com/a/b.git", direction: "fetch" },
      { name: "origin", url: "https://github.com/a/b.git", direction: "push" },
      { name: "upstream", url: "https://github.com/c/d.git", direction: "fetch" },
    ]);

    render(<RemoteSelector />);

    await waitFor(() => expect(screen.getByText("origin")).toBeInTheDocument());
    expect(screen.getByText("upstream")).toBeInTheDocument();
    expect(screen.getByDisplayValue("origin")).toBeInTheDocument();
  });

  it("invokes switch_remote when the selection changes", async () => {
    mockedInvoke.mockResolvedValueOnce([
      { name: "origin", url: "https://github.com/a/b.git", direction: "fetch" },
      { name: "upstream", url: "https://github.com/c/d.git", direction: "fetch" },
    ]);
    mockedInvoke.mockResolvedValueOnce(undefined); // switch_remote

    render(<RemoteSelector />);
    await waitFor(() => expect(screen.getByText("upstream")).toBeInTheDocument());

    fireEvent.change(screen.getByRole("combobox"), { target: { value: "upstream" } });

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("switch_remote", { remote: "upstream" })
    );
  });
});
