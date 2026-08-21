import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { StatusBadge } from "@/components/StatusBadge";
import { MetricCard } from "@/components/MetricCard";
import { CopyButton } from "@/components/CopyButton";
import { SearchBar } from "@/components/SearchBar";
import { LiveNetworkStrip } from "@/components/LiveNetworkStrip";

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

  it("renders LiveNetworkStrip with active telemetry", () => {
    const mockStats = {
      chain_id: "sprax-mainnet-1",
      latest_height: 8245920,
      latest_block_hash: "0x123",
      total_transactions: 1000,
      total_accounts: 50,
      active_validators_count: 100,
      total_bonded_tokens: "420,000,000 SPRX",
      avg_block_time_seconds: 1.5,
      current_tps: 1200,
      latest_state_root: "0x456",
    };

    render(
      <LiveNetworkStrip
        stats={mockStats}
        network="mainnet"
        onNavigate={() => {}}
      />
    );

    expect(screen.getByText(/sprax-mainnet-1/i)).toBeInTheDocument();
    expect(screen.getByText(/1.5s/i)).toBeInTheDocument();
  });
});
