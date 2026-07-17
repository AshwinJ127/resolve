import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import DirtyStateModal from "./DirtyStateModal";

const files = [
  { path: "a.txt", status: "M" },
  { path: "b.txt", status: "??" },
];

describe("DirtyStateModal", () => {
  it("renders nothing when closed", () => {
    const { container } = render(
      <DirtyStateModal open={false} files={files} onCommit={vi.fn()} onStash={vi.fn()} onCancel={vi.fn()} />
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("lists each changed file when open", () => {
    render(
      <DirtyStateModal open={true} files={files} onCommit={vi.fn()} onStash={vi.fn()} onCancel={vi.fn()} />
    );
    expect(screen.getByText("a.txt")).toBeInTheDocument();
    expect(screen.getByText("b.txt")).toBeInTheDocument();
  });

  it("calls onCommit when 'Commit changes, then sync' is clicked", () => {
    const onCommit = vi.fn();
    render(
      <DirtyStateModal open={true} files={files} onCommit={onCommit} onStash={vi.fn()} onCancel={vi.fn()} />
    );
    fireEvent.click(screen.getByText("Commit changes, then sync"));
    expect(onCommit).toHaveBeenCalledOnce();
  });

  it("calls onStash when 'Stash changes and sync' is clicked", () => {
    const onStash = vi.fn();
    render(
      <DirtyStateModal open={true} files={files} onCommit={vi.fn()} onStash={onStash} onCancel={vi.fn()} />
    );
    fireEvent.click(screen.getByText("Stash changes and sync"));
    expect(onStash).toHaveBeenCalledOnce();
  });

  it("calls onCancel when Cancel is clicked", () => {
    const onCancel = vi.fn();
    render(
      <DirtyStateModal open={true} files={files} onCommit={vi.fn()} onStash={vi.fn()} onCancel={onCancel} />
    );
    fireEvent.click(screen.getByText("Cancel"));
    expect(onCancel).toHaveBeenCalledOnce();
  });
});
