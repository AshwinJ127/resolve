import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import NewCommitModal from "./NewCommitModal";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

describe("NewCommitModal", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("renders nothing when closed", () => {
    const { container } = render(<NewCommitModal open={false} onClose={vi.fn()} onSubmit={vi.fn()} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("loads pending changes on open and selects them all by default", async () => {
    mockedInvoke.mockResolvedValueOnce([
      { path: "a.txt", status: "M" },
      { path: "b.txt", status: "??" },
    ]);

    render(<NewCommitModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} />);

    expect(mockedInvoke).toHaveBeenCalledWith("get_pending_changes");
    await waitFor(() => expect(screen.getByText("2 file(s) selected")).toBeInTheDocument());
  });

  it("invokes commit_selection with the message and selected files on commit", async () => {
    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]); // get_pending_changes
    mockedInvoke.mockResolvedValueOnce("commit ok"); // commit_selection

    const onSubmit = vi.fn();
    render(<NewCommitModal open={true} onClose={vi.fn()} onSubmit={onSubmit} />);

    await waitFor(() => expect(screen.getByText("1 file(s) selected")).toBeInTheDocument());

    fireEvent.change(screen.getByPlaceholderText(/feat: implemented/), {
      target: { value: "Fix the thing" },
    });
    fireEvent.click(screen.getByText("Commit Changes"));

    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("commit_selection", {
        message: "Fix the thing",
        files: ["a.txt"],
      })
    );
    await waitFor(() => expect(onSubmit).toHaveBeenCalledOnce());
  });

  it("disables the commit button when the message is empty", async () => {
    mockedInvoke.mockResolvedValueOnce([{ path: "a.txt", status: "M" }]);
    render(<NewCommitModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Commit Changes")).toBeDisabled());
  });
});
