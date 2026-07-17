import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import BranchList from "./BranchList";

const branches = [
  { name: "main", current: true, ahead: 2, behind: 1, author: "Ashwin", date: "2026-01-01", message: "Initial commit" },
  { name: "feature-x", current: false, ahead: 0, behind: 0, author: "Ashwin", date: "2026-01-02", message: "WIP" },
];

describe("BranchList", () => {
  it("shows the branch count", () => {
    const testBranches = [
      { name: "main", current: true, ahead: 5, behind: 3, author: "Ashwin", date: "2026-01-01", message: "Initial commit" },
    ];
    render(<BranchList branches={testBranches} />);
    expect(screen.getByText("1")).toBeInTheDocument();
  });

  it("marks the current branch with a HEAD badge", () => {
    render(<BranchList branches={branches} />);
    expect(screen.getByText("HEAD")).toBeInTheDocument();
    expect(screen.getByText("main")).toBeInTheDocument();
    expect(screen.getByText("feature-x")).toBeInTheDocument();
  });

  it("shows ahead/behind counts for a branch that has them", () => {
    render(<BranchList branches={branches} />);
    expect(screen.getByTitle("2 commits ahead")).toBeInTheDocument();
    expect(screen.getByTitle("1 commits behind")).toBeInTheDocument();
  });

  it("renders an empty table when there are no branches", () => {
    render(<BranchList branches={[]} />);
    expect(screen.getByText("0")).toBeInTheDocument();
  });
});
