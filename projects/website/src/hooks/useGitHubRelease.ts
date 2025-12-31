"use client";

import { useState, useEffect } from "react";

interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
  download_count: number;
}

interface GitHubRelease {
  tag_name: string;
  name: string;
  published_at: string;
  html_url: string;
  assets: ReleaseAsset[];
}

interface ReleaseInfo {
  version: string;
  publishedAt: string;
  releaseUrl: string;
  downloadUrl: string | null;
  fileName: string | null;
  fileSize: string | null;
  downloadCount: number;
  isLoading: boolean;
  error: string | null;
}

const GITHUB_REPO = "lhanks/syslens";
const GITHUB_RELEASES_URL = `https://github.com/${GITHUB_REPO}/releases`;
const CACHE_KEY = "syslens-release-info";
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function useGitHubRelease(): ReleaseInfo {
  const [releaseInfo, setReleaseInfo] = useState<ReleaseInfo>({
    version: "",
    publishedAt: "",
    releaseUrl: GITHUB_RELEASES_URL,
    downloadUrl: null,
    fileName: null,
    fileSize: null,
    downloadCount: 0,
    isLoading: true,
    error: null,
  });

  useEffect(() => {
    async function fetchRelease() {
      // Check cache first
      try {
        const cached = localStorage.getItem(CACHE_KEY);
        if (cached) {
          const { data, timestamp } = JSON.parse(cached);
          if (Date.now() - timestamp < CACHE_DURATION) {
            setReleaseInfo({ ...data, isLoading: false, error: null });
            return;
          }
        }
      } catch {
        // Ignore cache errors
      }

      try {
        const response = await fetch(
          `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
          {
            headers: {
              Accept: "application/vnd.github.v3+json",
            },
          }
        );

        if (!response.ok) {
          throw new Error(`GitHub API error: ${response.status}`);
        }

        const release: GitHubRelease = await response.json();

        // Find the Windows installer (.msi or .exe)
        const installerAsset = release.assets.find(
          (asset) =>
            asset.name.endsWith(".msi") ||
            asset.name.endsWith(".exe") ||
            asset.name.endsWith("_x64-setup.exe") ||
            asset.name.endsWith("_x64_en-US.msi")
        );

        // Calculate total downloads across all assets
        const totalDownloads = release.assets.reduce(
          (sum, asset) => sum + asset.download_count,
          0
        );

        const info: ReleaseInfo = {
          version: release.tag_name.replace(/^v/, ""),
          publishedAt: formatDate(release.published_at),
          releaseUrl: release.html_url,
          downloadUrl: installerAsset?.browser_download_url || null,
          fileName: installerAsset?.name || null,
          fileSize: installerAsset ? formatBytes(installerAsset.size) : null,
          downloadCount: totalDownloads,
          isLoading: false,
          error: null,
        };

        // Cache the result
        try {
          localStorage.setItem(
            CACHE_KEY,
            JSON.stringify({ data: info, timestamp: Date.now() })
          );
        } catch {
          // Ignore cache errors
        }

        setReleaseInfo(info);
      } catch (err) {
        // On error, ensure we have valid fallback URLs
        setReleaseInfo({
          version: "",
          publishedAt: "",
          releaseUrl: GITHUB_RELEASES_URL,
          downloadUrl: null,
          fileName: null,
          fileSize: null,
          downloadCount: 0,
          isLoading: false,
          error: err instanceof Error ? err.message : "Failed to fetch release",
        });
      }
    }

    fetchRelease();
  }, []);

  return releaseInfo;
}
