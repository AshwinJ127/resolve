import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import FileSelector from "./FileSelector";

const files = [
  { path: "src/a.txt", status: "M" },
  { path: "src/b.txt", status: "??" },
  { path: "src/c.txt", status: "D" },
];

describe("FileSelector", () => {
  it("shows the changed file count", () => {
    render(
      <FileSelector files={files} selected={new Set()} onToggle={vi.fn()} onToggleAll={vi.fn()} />
    );
    expect(screen.getByText("3 Changed Files")).toBeInTheDocument();
  });

  it("calls onToggle with the file path when a row is clicked", () => {
    const onToggle = vi.fn();
    render(
      <FileSelector files={files} selected={new Set()} onToggle={onToggle} onToggleAll={vi.fn()} />
    );
    fireEvent.click(screen.getByText("a.txt"));
    expect(onToggle).toHaveBeenCalledWith("src/a.txt");
  });

  it("shows 'Select All' when nothing is selected, 'Deselect All' when everything is", () => {
    const { rerender } = render(
      <FileSelector files={files} selected={new Set()} onToggle={vi.fn()} onToggleAll={vi.fn()} />
    );
    expect(screen.getByText("Select All")).toBeInTheDocument();

    rerender(
      <FileSelector
        files={files}
        selected={new Set(files.map((f) => f.path))}
        onToggle={vi.fn()}
        onToggleAll={vi.fn()}
      />
    );
    expect(screen.getByText("Deselect All")).toBeInTheDocument();
  });

  it("calls onToggleAll when the select-all button is clicked", () => {
    const onToggleAll = vi.fn();
    render(
      <FileSelector files={files} selected={new Set()} onToggle={vi.fn()} onToggleAll={onToggleAll} />
    );
    fireEvent.click(screen.getByText("Select All"));
    expect(onToggleAll).toHaveBeenCalledOnce();
  });
});
