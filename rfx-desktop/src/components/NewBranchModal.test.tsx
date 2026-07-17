import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import NewBranchModal from "./NewBranchModal";

describe("NewBranchModal", () => {
  it("renders nothing when closed", () => {
    const { container } = render(<NewBranchModal open={false} onClose={vi.fn()} onSubmit={vi.fn()} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("replaces spaces with dashes as the user types", () => {
    render(<NewBranchModal open={true} onClose={vi.fn()} onSubmit={vi.fn()} />);
    const input = screen.getByPlaceholderText("feature/my-new-feature") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "my new branch" } });
    expect(input.value).toBe("my-new-branch");
  });

  it("calls onSubmit with the branch name when the form is submitted", () => {
    const onSubmit = vi.fn();
    render(<NewBranchModal open={true} onClose={vi.fn()} onSubmit={onSubmit} />);
    const input = screen.getByPlaceholderText("feature/my-new-feature");
    fireEvent.change(input, { target: { value: "feature-x" } });
    fireEvent.click(screen.getByText("Create Branch"));
    expect(onSubmit).toHaveBeenCalledWith("feature-x");
  });

  it("does not call onSubmit when the name is empty", () => {
    const onSubmit = vi.fn();
    render(<NewBranchModal open={true} onClose={vi.fn()} onSubmit={onSubmit} />);
    expect(screen.getByText("Create Branch")).toBeDisabled();
  });

  it("calls onClose when the X button is clicked", () => {
    const onClose = vi.fn();
    render(<NewBranchModal open={true} onClose={onClose} onSubmit={vi.fn()} />);
    fireEvent.click(screen.getByText("Cancel"));
    expect(onClose).toHaveBeenCalledOnce();
  });
});
