import Image from "next/image";

export default function Home() {
  return (
    <div className="min-h-screen">
      {/* Navigation */}
      <nav className="fixed top-0 left-0 right-0 z-50 bg-[#0F172A]/80 backdrop-blur-md border-b border-[#334155]">
        <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Image src="/logo-icon.svg" alt="Syslens" width={32} height={32} />
            <span className="text-xl font-semibold">Syslens</span>
          </div>
          <div className="flex items-center gap-6">
            <a href="#features" className="text-[#94A3B8] hover:text-white transition-colors">Features</a>
            <a href="https://github.com/syslens/syslens" className="text-[#94A3B8] hover:text-white transition-colors">GitHub</a>
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
            A modern desktop system information dashboard. Real-time monitoring
            for CPU, memory, GPU, storage, network, and processes.
          </p>
          <div className="flex justify-center gap-4">
            <a
              href="#download"
              className="px-8 py-4 bg-gradient-to-r from-[#3B82F6] to-[#8B5CF6] hover:opacity-90 text-white rounded-xl font-semibold transition-opacity glow"
            >
              Download for Windows
            </a>
            <a
              href="https://github.com/syslens/syslens"
              className="px-8 py-4 border border-[#334155] hover:border-[#3B82F6] text-white rounded-xl font-semibold transition-colors"
            >
              View on GitHub
            </a>
          </div>
        </div>
      </section>

      {/* Screenshot/Preview Section */}
      <section className="py-16 px-6">
        <div className="max-w-5xl mx-auto">
          <div className="card p-2 glow">
            <div className="bg-[#020617] rounded-lg aspect-video flex items-center justify-center">
              <p className="text-[#94A3B8]">App Screenshot Preview</p>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" className="py-20 px-6">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-4">Everything at a glance</h2>
          <p className="text-[#94A3B8] text-center mb-12 max-w-2xl mx-auto">
            Syslens provides comprehensive system monitoring with a clean, modern interface.
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
              <h3 className="text-xl font-semibold mb-2">GPU Information</h3>
              <p className="text-[#94A3B8]">
                Graphics card details, VRAM, driver version with links to latest drivers.
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
              <h3 className="text-xl font-semibold mb-2">Network Stats</h3>
              <p className="text-[#94A3B8]">
                Real-time bandwidth graphs, adapter details, IP configuration, and DNS settings.
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
                View running processes, CPU/memory usage, and manage applications.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Tech Stack */}
      <section className="py-16 px-6 bg-[#020617]">
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
        </div>
      </section>

      {/* Download Section */}
      <section id="download" className="py-20 px-6">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-3xl font-bold mb-4">Ready to get started?</h2>
          <p className="text-[#94A3B8] mb-8">
            Download Syslens for free. Available for Windows 10/11.
          </p>
          <div className="flex justify-center gap-4 flex-wrap">
            <a
              href="#"
              className="px-8 py-4 bg-gradient-to-r from-[#3B82F6] to-[#8B5CF6] hover:opacity-90 text-white rounded-xl font-semibold transition-opacity"
            >
              Download .exe Installer
            </a>
            <a
              href="#"
              className="px-8 py-4 border border-[#334155] hover:border-[#3B82F6] text-white rounded-xl font-semibold transition-colors"
            >
              Download .msi Package
            </a>
          </div>
          <p className="text-sm text-[#64748B] mt-6">
            v1.0.0 &bull; Windows 10/11 &bull; ~15 MB
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
            <a href="https://github.com/syslens/syslens" className="text-[#94A3B8] hover:text-white transition-colors">
              GitHub
            </a>
            <a href="#" className="text-[#94A3B8] hover:text-white transition-colors">
              Releases
            </a>
            <a href="#" className="text-[#94A3B8] hover:text-white transition-colors">
              Documentation
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}
