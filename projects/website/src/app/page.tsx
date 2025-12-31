"use client";

import Image from "next/image";
import { useState } from "react";
import { useGitHubRelease } from "../hooks/useGitHubRelease";

// Screenshot data for the gallery
const screenshots = [
  { src: "/screenshots/hardware-page.png", alt: "Hardware Dashboard", title: "Hardware Dashboard", desc: "CPU, GPU, memory specs with vendor badges", color: "#3B82F6" },
  { src: "/screenshots/system-page.png", alt: "System Overview", title: "System Overview", desc: "OS info, uptime, and BIOS details", color: "#8B5CF6" },
  { src: "/screenshots/processes-page.png", alt: "Process Manager", title: "Process Manager", desc: "App icons, grouping, and resource usage", color: "#10B981" },
  { src: "/screenshots/services-page.png", alt: "Services Manager", title: "Services Manager", desc: "Windows services with status and filters", color: "#A855F7" },
  { src: "/screenshots/network-page.png", alt: "Network Adapters", title: "Network Adapters", desc: "Per-adapter graphs and IP configuration", color: "#F59E0B" },
  { src: "/screenshots/storage-page.png", alt: "Storage Health", title: "Storage Health", desc: "Disk usage, volumes, and S.M.A.R.T. status", color: "#06B6D4" },
];

export default function Home() {
  const [lightboxImage, setLightboxImage] = useState<string | null>(null);
  const [lightboxAlt, setLightboxAlt] = useState<string>("");
  const release = useGitHubRelease();

  const openLightbox = (src: string, alt: string) => {
    setLightboxImage(src);
    setLightboxAlt(alt);
  };

  const closeLightbox = () => {
    setLightboxImage(null);
    setLightboxAlt("");
  };

  return (
    <div className="min-h-screen">
      {/* Lightbox Modal */}
      {lightboxImage && (
        <div
          className="fixed inset-0 z-[100] bg-black/90 flex items-center justify-center p-4 cursor-pointer"
          onClick={closeLightbox}
        >
          <button
            className="absolute top-4 right-4 text-white text-4xl hover:text-[#3B82F6] transition-colors"
            onClick={closeLightbox}
            aria-label="Close"
          >
            &times;
          </button>
          <div className="max-w-[95vw] max-h-[95vh] cursor-pointer">
            <Image
              src={lightboxImage}
              alt={lightboxAlt}
              width={1920}
              height={1080}
              className="max-w-full max-h-[90vh] object-contain rounded-lg shadow-2xl"
              priority
            />
            <p className="text-center text-white/80 mt-4 text-lg">{lightboxAlt}</p>
          </div>
        </div>
      )}

      {/* Navigation */}
      <nav className="fixed top-0 left-0 right-0 z-50 bg-[#0F172A]/80 backdrop-blur-md border-b border-[#334155]">
        <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Image src="/logo-icon.svg" alt="Syslens" width={32} height={32} />
            <span className="text-xl font-semibold">Syslens</span>
          </div>
          <div className="flex items-center gap-6">
            <a href="#features" className="text-[#94A3B8] hover:text-white transition-colors">Features</a>
            <a href="#gallery" className="text-[#94A3B8] hover:text-white transition-colors">Gallery</a>
            <a href="https://github.com/lhanks/syslens" className="text-[#94A3B8] hover:text-white transition-colors">GitHub</a>
            <a
              href="#download"
              className="px-4 py-2 bg-[#3B82F6] hover:bg-[#60A5FA] text-white rounded-lg transition-colors"
            >
              Download
            </a>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="pt-32 pb-20 px-6">
        <div className="max-w-4xl mx-auto text-center">
          <div className="mb-8">
            <Image
              src="/logo-full.svg"
              alt="Syslens"
              width={200}
              height={60}
              className="mx-auto"
            />
          </div>
          <h1 className="text-5xl md:text-6xl font-bold mb-6">
            <span className="gradient-text">Clarity</span> for your system
          </h1>
          <p className="text-xl text-[#94A3B8] mb-10 max-w-2xl mx-auto">
            A modern, customizable desktop system dashboard. Real-time monitoring
            with dockable panels, hardware enrichment, and a fully flexible layout.
          </p>
          <div className="flex justify-center gap-4">
            <a
              href={release.downloadUrl || "#download"}
              className="px-8 py-4 bg-gradient-to-r from-[#3B82F6] to-[#8B5CF6] hover:opacity-90 text-white rounded-xl font-semibold transition-opacity glow flex items-center gap-3"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              Download for Windows
            </a>
            <a
              href="https://github.com/lhanks/syslens"
              className="px-8 py-4 border border-[#334155] hover:border-[#3B82F6] text-white rounded-xl font-semibold transition-colors"
            >
              View on GitHub
            </a>
          </div>
        </div>
      </section>

      {/* Screenshot/Preview Section */}
      <section className="py-16 px-6 overflow-hidden">
        <div className="max-w-6xl mx-auto">
          {/* Main Screenshot with perspective effect */}
          <div className="relative" style={{ perspective: '1500px' }}>
            <div
              className="relative mx-auto max-w-4xl"
              style={{
                transform: 'rotateX(5deg) rotateY(-5deg)',
                transformStyle: 'preserve-3d'
              }}
            >
              {/* Main screenshot - clickable */}
              <div
                className="card p-2 glow shadow-2xl cursor-pointer hover:scale-[1.02] transition-transform"
                onClick={() => openLightbox("/screenshots/hardware-page.png", "Hardware Dashboard")}
              >
                <Image
                  src="/screenshots/hardware-page.png"
                  alt="Syslens Hardware Dashboard"
                  width={1280}
                  height={800}
                  className="rounded-lg"
                  priority
                />
                <div className="absolute inset-0 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity bg-black/20 rounded-lg">
                  <span className="bg-white/90 text-black px-4 py-2 rounded-lg font-medium">Click to enlarge</span>
                </div>
              </div>

              {/* Secondary screenshots fading into distance */}
              <div
                className="absolute -right-32 top-16 w-72 opacity-40 blur-[1px] pointer-events-none"
                style={{
                  transform: 'translateZ(-100px) rotateY(15deg)',
                  transformStyle: 'preserve-3d'
                }}
              >
                <div className="card p-1 shadow-xl">
                  <Image
                    src="/screenshots/processes-page.png"
                    alt="Syslens Processes"
                    width={640}
                    height={400}
                    className="rounded"
                  />
                </div>
              </div>

              <div
                className="absolute -left-24 top-24 w-64 opacity-30 blur-[2px] pointer-events-none"
                style={{
                  transform: 'translateZ(-150px) rotateY(-20deg)',
                  transformStyle: 'preserve-3d'
                }}
              >
                <div className="card p-1 shadow-xl">
                  <Image
                    src="/screenshots/services-page.png"
                    alt="Syslens Services"
                    width={640}
                    height={400}
                    className="rounded"
                  />
                </div>
              </div>
            </div>
          </div>

          {/* Caption */}
          <p className="text-center text-[#64748B] mt-8 text-sm">
            Hardware dashboard showing CPU, GPU, memory, and system details
          </p>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" className="py-20 px-6">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-4">Everything at a glance</h2>
          <p className="text-[#94A3B8] text-center mb-12 max-w-2xl mx-auto">
            Comprehensive system monitoring with a clean, modern interface and fully customizable layout.
          </p>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* CPU */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#3B82F6]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#3B82F6]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">CPU Monitoring</h3>
              <p className="text-[#94A3B8]">
                Real-time CPU usage, per-core metrics, temperature, and clock speeds with historical graphs.
              </p>
            </div>

            {/* Memory */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#8B5CF6]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#8B5CF6]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Memory Details</h3>
              <p className="text-[#94A3B8]">
                RAM usage, vendor info, speed, and individual module details with XMP detection.
              </p>
            </div>

            {/* GPU */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#10B981]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#10B981]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">GPU &amp; Hardware Enrichment</h3>
              <p className="text-[#94A3B8]">
                Auto-fetched specs, images, and documentation from TechPowerUp, Wikipedia, and manufacturer sites.
              </p>
            </div>

            {/* Storage */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#06B6D4]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#06B6D4]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Storage Health</h3>
              <p className="text-[#94A3B8]">
                Disk usage, S.M.A.R.T. health monitoring, and volume information.
              </p>
            </div>

            {/* Network */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#F59E0B]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#F59E0B]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Network Control</h3>
              <p className="text-[#94A3B8]">
                Per-adapter bandwidth graphs, enable/disable adapters, IP configuration, and DNS settings.
              </p>
            </div>

            {/* Processes */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#EF4444]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#EF4444]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Process Manager</h3>
              <p className="text-[#94A3B8]">
                Running processes with icons, groupable tree view, CPU/memory graphs, and application management.
              </p>
            </div>

            {/* Services */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#A855F7]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#A855F7]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Services Manager</h3>
              <p className="text-[#94A3B8]">
                View Windows services with status indicators, startup types, search, and filtering by state.
              </p>
            </div>

            {/* Docking Layout */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#EC4899]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#EC4899]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Dockable Panels</h3>
              <p className="text-[#94A3B8]">
                Fully customizable layout with resizable dock regions, stackable tabs, and floating windows.
              </p>
            </div>

            {/* Vendor Badges */}
            <div className="card p-6 transition-all">
              <div className="w-12 h-12 rounded-xl bg-[#14B8A6]/20 flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-[#14B8A6]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 3.138 3.42 3.42 0 00.806 1.946 3.42 3.42 0 010 4.438 3.42 3.42 0 00-.806 1.946 3.42 3.42 0 01-3.138 3.138 3.42 3.42 0 00-1.946.806 3.42 3.42 0 01-4.438 0 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-3.138-3.138 3.42 3.42 0 00-.806-1.946 3.42 3.42 0 010-4.438 3.42 3.42 0 00.806-1.946 3.42 3.42 0 013.138-3.138z" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Vendor Recognition</h3>
              <p className="text-[#94A3B8]">
                Beautiful brand-colored badges for hardware vendors with quick links to support pages.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Screenshot Gallery */}
      <section id="gallery" className="py-20 px-6 bg-[#020617]">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-4">See it in action</h2>
          <p className="text-[#94A3B8] text-center mb-4 max-w-2xl mx-auto">
            Explore every corner of the application with these feature screenshots.
          </p>
          <p className="text-[#64748B] text-center mb-12 text-sm">
            Click any image to view full size
          </p>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {screenshots.map((shot, index) => (
              <div key={index} className="group">
                <div
                  className="card p-2 transition-all cursor-pointer hover:scale-[1.02]"
                  style={{ borderColor: 'transparent' }}
                  onMouseEnter={(e) => (e.currentTarget.style.borderColor = `${shot.color}80`)}
                  onMouseLeave={(e) => (e.currentTarget.style.borderColor = 'transparent')}
                  onClick={() => openLightbox(shot.src, shot.alt)}
                >
                  <div className="relative">
                    <Image
                      src={shot.src}
                      alt={shot.alt}
                      width={640}
                      height={400}
                      className="rounded-lg"
                    />
                    <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity bg-black/30 rounded-lg">
                      <span className="bg-white/90 text-black px-3 py-1.5 rounded-lg text-sm font-medium">
                        View full size
                      </span>
                    </div>
                  </div>
                </div>
                <h3 className="text-lg font-semibold mt-3 text-center">{shot.title}</h3>
                <p className="text-sm text-[#64748B] text-center">{shot.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Tech Stack */}
      <section className="py-16 px-6">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-2xl font-bold mb-8">Built with modern technology</h2>
          <div className="flex justify-center gap-8 flex-wrap">
            <div className="text-[#94A3B8]">
              <span className="font-mono text-[#3B82F6]">Tauri 2.0</span> + Rust
            </div>
            <div className="text-[#94A3B8]">
              <span className="font-mono text-[#8B5CF6]">Angular 21</span> + TypeScript
            </div>
            <div className="text-[#94A3B8]">
              <span className="font-mono text-[#06B6D4]">Tailwind CSS</span>
            </div>
          </div>
          <p className="text-[#64748B] mt-6 text-sm">
            Native performance. Tiny footprint. No Electron bloat.
          </p>
        </div>
      </section>

      {/* Download Section */}
      <section id="download" className="py-20 px-6">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-3xl font-bold mb-4">Ready to get started?</h2>
          <p className="text-[#94A3B8] mb-8">
            Download Syslens for free. Open source and available for Windows 10/11.
          </p>

          {/* Version badge */}
          {release.version && (
            <div className="mb-6">
              <span className="inline-flex items-center gap-2 px-4 py-2 bg-[#1E293B] rounded-full text-sm">
                <span className="w-2 h-2 bg-[#10B981] rounded-full animate-pulse"></span>
                <span className="text-[#94A3B8]">Latest:</span>
                <span className="text-white font-mono">v{release.version}</span>
                {release.publishedAt && (
                  <span className="text-[#64748B]">&bull; {release.publishedAt}</span>
                )}
              </span>
            </div>
          )}

          <div className="flex justify-center gap-4 flex-wrap">
            <a
              href={release.downloadUrl || release.releaseUrl}
              className="px-8 py-4 bg-gradient-to-r from-[#3B82F6] to-[#8B5CF6] hover:opacity-90 text-white rounded-xl font-semibold transition-opacity flex items-center gap-3"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              {release.isLoading ? "Loading..." : release.version ? `Download v${release.version}` : "View Releases"}
            </a>
            <a
              href="https://github.com/lhanks/syslens/releases"
              className="px-8 py-4 border border-[#334155] hover:border-[#3B82F6] text-white rounded-xl font-semibold transition-colors"
            >
              All Releases
            </a>
          </div>

          <p className="text-sm text-[#64748B] mt-6">
            Windows 10/11 &bull; 64-bit
            {release.fileSize && <> &bull; {release.fileSize}</>}
            {release.downloadCount > 0 && <> &bull; {release.downloadCount.toLocaleString()} downloads</>}
          </p>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-8 px-6 border-t border-[#334155]">
        <div className="max-w-6xl mx-auto flex flex-col md:flex-row justify-between items-center gap-4">
          <div className="flex items-center gap-3">
            <Image src="/logo-icon.svg" alt="Syslens" width={24} height={24} />
            <span className="text-[#94A3B8]">&copy; 2025 Syslens</span>
          </div>
          <div className="flex items-center gap-6">
            <a href="https://github.com/lhanks/syslens" className="text-[#94A3B8] hover:text-white transition-colors">
              GitHub
            </a>
            <a href="https://github.com/lhanks/syslens/releases" className="text-[#94A3B8] hover:text-white transition-colors">
              Releases
            </a>
            <a href="https://github.com/lhanks/syslens#readme" className="text-[#94A3B8] hover:text-white transition-colors">
              Documentation
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}
