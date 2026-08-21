import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { StatusBadge } from "@/components/StatusBadge";
import { MetricCard } from "@/components/MetricCard";
import { CopyButton } from "@/components/CopyButton";
import { SearchBar } from "@/components/SearchBar";

describe("UI Components", () => {
  it("renders StatusBadge correctly for confirmed and failed states", () => {
    const { rerender } = render(<StatusBadge status="confirmed" />);
    expect(screen.getByText("Confirmed")).toBeInTheDocument();

    rerender(<StatusBadge status="failed" />);
    expect(screen.getByText("Failed")).toBeInTheDocument();
  });

  it("renders MetricCard with labels, values, and sub-values", () => {
    render(
      <MetricCard
        label="Block Height"
        value="#8,245,920"
        subValue="1-block finality"
      />
    );
    expect(screen.getByText("Block Height")).toBeInTheDocument();
    expect(screen.getByText("#8,245,920")).toBeInTheDocument();
    expect(screen.getByText("1-block finality")).toBeInTheDocument();
  });

  it("renders CopyButton and handles click", async () => {
    render(<CopyButton text="0x12345678" />);
    const btn = screen.getByRole("button", { name: /copy to clipboard/i });
    expect(btn).toBeInTheDocument();
    fireEvent.click(btn);
  });

  it("renders SearchBar and fires onSearch on submit", () => {
    const onSearch = vi.fn();
    render(<SearchBar onSearch={onSearch} />);
    const input = screen.getByPlaceholderText(/search by address/i);
    fireEvent.change(input, { target: { value: "8245920" } });
    const submitBtn = screen.getByRole("button", { name: /search/i });
    fireEvent.click(submitBtn);

    expect(onSearch).toHaveBeenCalledWith("8245920");
  });
});
