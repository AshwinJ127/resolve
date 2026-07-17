import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import SwitchBranchModal from "./SwitchBranchModal";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const branches = [
  { name: "main", current: true, ahead: 0, behind: 0, author: "Test", date: "2026-01-01", message: "init" },
  { name: "feature", current: false, ahead: 0, behind: 0, author: "Test", date: "2026-01-01", message: "wip" },
];

describe("SwitchBranchModal", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("renders nothing when closed", () => {
    const { container } = render(
      <SwitchBranchModal open={false} onClose={vi.fn()} onSubmit={vi.fn()} branches={branches} />
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("excludes the current branch from the switch-to list", async () => {
    mockedInvoke.mockResolvedValueOnce([]); // get_pending_changes: clean

    render(<SwitchBranchModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} branches={branches} />);

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("get_pending_changes"));
    expect(screen.queryByRole("option", { name: "main" })).not.toBeInTheDocument();
    expect(screen.getByRole("option", { name: "feature" })).toBeInTheDocument();
  });

  it("switches directly when the working directory is clean", async () => {
    mockedInvoke.mockResolvedValueOnce([]); // get_pending_changes
    mockedInvoke.mockResolvedValueOnce(undefined); // switch_branch

    const onSubmit = vi.fn();
    render(<SwitchBranchModal open={true} onClose={vi.fn()} onSubmit={onSubmit} branches={branches} />);

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("get_pending_changes"));
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "feature" } });
    fireEvent.click(screen.getByRole("button", { name: /Switch Branch/i }));

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("switch_branch", { name: "feature" })
    );
    await waitFor(() => expect(onSubmit).toHaveBeenCalledOnce());
  });

  it("requires picking stash before switching when there are uncommitted changes", async () => {
    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]); // get_pending_changes: dirty

    render(<SwitchBranchModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} branches={branches} />);

    await waitFor(() => expect(screen.getByText("You have uncommitted changes")).toBeInTheDocument());
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "feature" } });
    expect(screen.getByRole("button", { name: /Switch Branch/i })).toBeDisabled();
  });

  it("stashes, switches, and pops when 'stash' is chosen with dirty changes", async () => {
    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]); // get_pending_changes
    mockedInvoke.mockResolvedValueOnce(undefined); // stash_changes
    mockedInvoke.mockResolvedValueOnce(undefined); // switch_branch
    mockedInvoke.mockResolvedValueOnce(undefined); // pop_stash

    const onSubmit = vi.fn();
    render(<SwitchBranchModal open={true} onClose={vi.fn()} onSubmit={onSubmit} branches={branches} />);

    await waitFor(() => expect(screen.getByText("You have uncommitted changes")).toBeInTheDocument());
    fireEvent.click(screen.getByText("Stash changes and switch"));
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "feature" } });
    fireEvent.click(screen.getByText("Stash & Switch"));

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("stash_changes"));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("switch_branch", { name: "feature" }));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("pop_stash"));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledOnce());
  });
});
