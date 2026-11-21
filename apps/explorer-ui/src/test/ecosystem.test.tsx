import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor, cleanup } from "@testing-library/react";
import { EcosystemPage } from "@/pages/EcosystemPage";
import { ExplorerApiService, apiService } from "@/services/api";
import { MOCK_ECOSYSTEM_PROJECTS, MOCK_MARKET_ASSETS } from "@/services/mockData";

afterEach(() => { cleanup(); vi.restoreAllMocks(); });

describe("ecosystem pages", () => {
  beforeEach(() => {
    vi.spyOn(apiService, "getEcosystemProjects").mockResolvedValue(MOCK_ECOSYSTEM_PROJECTS);
    vi.spyOn(apiService, "getMarketAssets").mockResolvedValue(MOCK_MARKET_ASSETS);
  });

  it("switches content when the header changes routes without unmounting", async () => {
    const view = render(<EcosystemPage currency="USD" initialTab="/ecosystem" />);
    await screen.findByText("SpraxSwap DEX");
    view.rerender(<EcosystemPage currency="USD" initialTab="/discover" />);
    expect(screen.getByRole("heading", {name:"Discover Showcase"})).toBeInTheDocument();
    view.rerender(<EcosystemPage currency="USD" initialTab="/markets" />);
    expect(screen.getByRole("heading", {name:"SPRX Ecosystem Asset Tickers"})).toBeInTheDocument();
    view.rerender(<EcosystemPage currency="USD" initialTab="/ecosystem" />);
    expect(screen.getByRole("textbox",{name:"Search projects"})).toBeInTheDocument();
  });

  it("routes tabs and provides a recoverable empty search state", async () => {
    const navigate = vi.fn();
    render(<EcosystemPage currency="USD" onNavigate={navigate} />);
    await screen.findByText("SpraxSwap DEX");
    fireEvent.change(screen.getByRole("textbox",{name:"Search projects"}),{target:{value:"no-such-project"}});
    expect(screen.getByText(/No projects match/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button",{name:"Clear filters"}));
    expect(screen.getByText("SpraxSwap DEX")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button",{name:"Live Markets & Tickers"}));
    expect(navigate).toHaveBeenCalledWith("/markets");
  });

  it("handles a failed request and retries", async () => {
    vi.mocked(apiService.getEcosystemProjects).mockRejectedValueOnce(new Error("offline"));
    render(<EcosystemPage currency="USD" />);
    await screen.findByRole("alert");
    fireEvent.click(screen.getByRole("button",{name:"Retry"}));
    await screen.findByText("SpraxSwap DEX");
    await waitFor(() => expect(screen.queryByRole("alert")).not.toBeInTheDocument());
  });
});

describe("ecosystem API data", () => {
  it("maps backend directory records and does not substitute samples for an empty directory", async () => {
    const service = new ExplorerApiService();
    vi.spyOn(globalThis,"fetch").mockResolvedValueOnce(new Response(JSON.stringify({success:true,data:{sprax_ecosystem:[{id:"one",name:"Live project",category:"wallets",website_url:"https://example.com",is_verified:true}]}})))
      .mockResolvedValueOnce(new Response(JSON.stringify({success:true,data:{sprax_ecosystem:[]}})));
    expect(await service.getEcosystemProjects()).toEqual([expect.objectContaining({name:"Live project",category:"Wallets",website:"https://example.com"})]);
    expect(service.ecosystemIsPreview).toBe(false);
    expect(await service.getEcosystemProjects()).toEqual([]);
  });

  it("maps numeric market values and identifies offline sample prices", async () => {
    const service = new ExplorerApiService();
    vi.spyOn(globalThis,"fetch").mockResolvedValueOnce(new Response(JSON.stringify({data:[{id:"sprx",symbol:"SPRX",name:"SPRAX",price_usd:"1.25",price_change_pct_24h:"-2",volume_24h:"1000"}]})))
      .mockResolvedValueOnce(new Response("{}",{status:503}));
    expect(await service.getMarketAssets()).toEqual([expect.objectContaining({priceUsd:1.25,change24h:-2,volume24hUsd:1000})]);
    expect(service.marketsArePreview).toBe(false);
    await service.getMarketAssets();
    expect(service.marketsArePreview).toBe(true);
  });
});
