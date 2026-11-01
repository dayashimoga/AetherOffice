/**
 * Release & SBOM Presentation Coordinator (Sub-Phase 53.2.2)
 *
 * Reactive coordinator managing the Universal 5-Platform Installer Matrix,
 * CycloneDX v1.5 JSON & SPDX 2.3 bill of materials generation,
 * clean-room license compliance auditing, and cryptographic installer checksum verification.
 */

import {
  IPC,
  type InstallerArtifactDto,
  type InstallerMatrixManifestDto,
  type LicenseAuditReportDto,
  type TargetOsDto,
} from "../ipc";

export interface ReleaseAndSbomState {
  manifest: InstallerMatrixManifestDto | null;
  cyclonedxJson: string | null;
  spdxContent: string | null;
  licenseAudit: LicenseAuditReportDto | null;
  verifiedArtifacts: Record<string, boolean>;
  isLoading: boolean;
  errorMessage: string | null;
}

export class ReleaseAndSbomCoordinator {
  private state: ReleaseAndSbomState;
  private listeners: Array<(state: ReleaseAndSbomState) => void> = [];

  constructor() {
    this.state = {
      manifest: null,
      cyclonedxJson: null,
      spdxContent: null,
      licenseAudit: null,
      verifiedArtifacts: {},
      isLoading: false,
      errorMessage: null,
    };
  }

  public getState(): ReleaseAndSbomState {
    return { ...this.state, verifiedArtifacts: { ...this.state.verifiedArtifacts } };
  }

  public subscribe(listener: (state: ReleaseAndSbomState) => void): () => void {
    this.listeners.push(listener);
    listener(this.getState());
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  private notify(): void {
    const copy = this.getState();
    for (const listener of this.listeners) {
      listener(copy);
    }
  }

  /**
   * Fetches the universal 5-platform installer matrix and distribution manifest.
   */
  public async fetchManifest(): Promise<InstallerMatrixManifestDto> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const manifest = await IPC.releaseMatrixGetManifest();
      this.state.manifest = manifest;
      this.state.isLoading = false;
      this.notify();
      return manifest;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Generates CycloneDX v1.5 JSON Bill of Materials.
   */
  public async generateCycloneDx(): Promise<string> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const json = await IPC.sbomGenerateCycloneDx();
      this.state.cyclonedxJson = json;
      this.state.isLoading = false;
      this.notify();
      return json;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Generates SPDX 2.3 JSON or Tag-Value standard specification document.
   */
  public async generateSpdx(format: "json" | "tag-value" = "json"): Promise<string> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const content = await IPC.sbomGenerateSpdx(format);
      this.state.spdxContent = content;
      this.state.isLoading = false;
      this.notify();
      return content;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Runs clean-room permissive license compliance audit across all 12 domain crates
   * and dependencies, checking for GPL copyleft contamination.
   */
  public async runLicenseAudit(): Promise<LicenseAuditReportDto> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const report = await IPC.sbomAuditLicenses();
      this.state.licenseAudit = report;
      this.state.isLoading = false;
      this.notify();
      return report;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Verifies SHA-256 cryptographic checksum of an installer artifact.
   */
  public async verifyArtifact(
    artifactId: string,
    payload?: Uint8Array | number[],
    checksumHex?: string
  ): Promise<boolean> {
    this.state.errorMessage = null;

    try {
      const isVerified = await IPC.installerVerifyChecksum(artifactId, payload, checksumHex);
      this.state.verifiedArtifacts[artifactId] = isVerified;
      this.notify();
      return isVerified;
    } catch (err: unknown) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Returns all artifacts targeting a specific operating system.
   */
  public getArtifactsForPlatform(os: TargetOsDto): InstallerArtifactDto[] {
    if (!this.state.manifest) return [];
    return this.state.manifest.artifacts.filter((a) => a.platform === os);
  }

  /**
   * Finds artifact by its identifier.
   */
  public getArtifact(artifactId: string): InstallerArtifactDto | null {
    if (!this.state.manifest) return null;
    return this.state.manifest.artifacts.find((a) => a.id === artifactId) || null;
  }

  public clearError(): void {
    this.state.errorMessage = null;
    this.notify();
  }
}
