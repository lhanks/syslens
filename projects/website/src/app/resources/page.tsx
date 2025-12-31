"use client";

import Image from "next/image";
import Link from "next/link";

const resources = [
  {
    category: "Source Code",
    items: [
      {
        title: "GitHub Repository",
        url: "https://github.com/lhanks/syslens",
        description: "View source code, report issues, and contribute",
        icon: (
          <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
            <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
          </svg>
        ),
      },
      {
        title: "GitHub Releases",
        url: "https://github.com/lhanks/syslens/releases",
        description: "Download installer packages and release notes",
        icon: (
          <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
          </svg>
        ),
      },
    ],
  },
  {
    category: "Deployment",
    items: [
      {
        title: "Vercel Dashboard",
        url: "https://vercel.com/lhanks/syslens",
        description: "Website hosting and deployment status",
        icon: (
          <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
            <path d="M24 22.525H0l12-21.05 12 21.05z" />
          </svg>
        ),
      },
      {
        title: "Production Website",
        url: "https://syslens.vercel.app",
        description: "Live production deployment",
        icon: (
          <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
          </svg>
        ),
      },
    ],
  },
  {
    category: "Technology",
    items: [
      {
        title: "Tauri",
        url: "https://tauri.app",
        description: "Framework for building desktop apps with web technologies",
        icon: (
          <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
        ),
      },
      {
        title: "Angular",
        url: "https://angular.dev",
        description: "Frontend framework powering the UI",
        icon: (
          <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
            <path d="M9.931 12.645h4.138l-2.07-4.908m0-7.737L.68 3.982l1.726 14.771L12 24l9.596-5.242L23.32 3.984 11.999.001zm7.064 18.31h-2.638l-1.422-3.503H8.996l-1.422 3.504h-2.64L12 2.65z" />
          </svg>
        ),
      },
      {
        title: "Rust",
        url: "https://rust-lang.org",
        description: "Backend language for native performance",
        icon: (
          <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
            <path d="M23.835 11.703l-1.008-.623a13.367 13.367 0 00-.063-.723l.86-.755a.335.335 0 00-.088-.49l-.127-.07a13.473 13.473 0 00-.396-.667l.688-.874a.335.335 0 00-.17-.465l-.136-.056a13.178 13.178 0 00-.509-.593l.493-.974a.335.335 0 00-.247-.424l-.146-.035a13.256 13.256 0 00-.607-.502l.28-1.052a.335.335 0 00-.316-.37l-.15-.01a13.254 13.254 0 00-.69-.392l.057-1.1a.335.335 0 00-.377-.303l-.15.02a13.264 13.264 0 00-.754-.273l-.168-1.12a.335.335 0 00-.424-.23l-.145.05a13.216 13.216 0 00-.8-.14L17.5.748a.335.335 0 00-.462-.148l-.133.08c-.27-.02-.542-.032-.817-.032-.274 0-.547.011-.816.032l-.134-.08a.335.335 0 00-.462.148l-.38 1.03a13.252 13.252 0 00-.8.14l-.146-.05a.335.335 0 00-.424.23l-.167 1.12c-.257.084-.508.175-.755.273l-.15-.02a.335.335 0 00-.376.303l.056 1.1c-.234.126-.464.258-.69.392l-.149.01a.335.335 0 00-.316.37l.279 1.052c-.207.162-.41.33-.606.502l-.147.035a.335.335 0 00-.247.424l.493.974a13.18 13.18 0 00-.508.593l-.137.056a.335.335 0 00-.17.465l.688.874c-.144.218-.276.44-.396.667l-.127.07a.335.335 0 00-.088.49l.86.755c-.03.24-.05.48-.063.723l-1.008.623a.335.335 0 000 .574l1.008.623c.013.243.034.483.063.723l-.86.755a.335.335 0 00.088.49l.127.07c.12.228.251.449.396.667l-.688.874a.335.335 0 00.17.465l.137.056c.163.203.332.4.508.593l-.493.974a.335.335 0 00.247.424l.147.035c.196.172.399.34.606.502l-.279 1.052a.335.335 0 00.316.37l.149.01c.226.134.456.266.69.392l-.056 1.1a.335.335 0 00.376.303l.15-.02c.247.098.498.19.755.273l.167 1.12a.335.335 0 00.424.23l.146-.05c.263.055.53.102.8.14l.38 1.03a.335.335 0 00.462.148l.134-.08c.269.02.542.032.816.032.275 0 .547-.011.817-.032l.133.08a.335.335 0 00.462-.148l.38-1.03c.27-.038.537-.085.8-.14l.145.05a.335.335 0 00.424-.23l.168-1.12c.256-.084.507-.175.754-.273l.15.02a.335.335 0 00.377-.303l-.057-1.1c.234-.126.464-.258.69-.392l.15-.01a.335.335 0 00.316-.37l-.28-1.052c.208-.162.41-.33.607-.502l.146-.035a.335.335 0 00.247-.424l-.493-.974c.176-.193.346-.39.509-.593l.136-.056a.335.335 0 00.17-.465l-.688-.874c.145-.218.276-.44.396-.667l.127-.07a.335.335 0 00.088-.49l-.86-.755c.029-.24.05-.48.063-.723l1.008-.623a.335.335 0 000-.574zm-6.85 8.18a.936.936 0 11-.001-1.871.936.936 0 01.001 1.871zm.936-3.057a.748.748 0 01-.748.748h-1.122v.467c0 .412-.334.747-.747.747h-.375c-.413 0-.748-.335-.748-.747v-.467h-.375a.748.748 0 01-.747-.748V13.77h-.467a.748.748 0 01-.748-.748v-.374c0-.413.335-.748.748-.748h.467v-.374c0-.413.335-.748.747-.748h.375v-.467c0-.413.335-.748.748-.748h.374c.414 0 .748.335.748.748v.467h1.122c.413 0 .748.335.748.748v.374h.467c.413 0 .748.335.748.748v.374a.748.748 0 01-.748.748h-.467zm-6.556 3.057a.936.936 0 110-1.871.936.936 0 010 1.871z" />
          </svg>
        ),
      },
    ],
  },
  {
    category: "Documentation",
    items: [
      {
        title: "README",
        url: "https://github.com/lhanks/syslens#readme",
        description: "Getting started and project overview",
        icon: (
          <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        ),
      },
      {
        title: "Issues & Feature Requests",
        url: "https://github.com/lhanks/syslens/issues",
        description: "Report bugs or request new features",
        icon: (
          <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        ),
      },
    ],
  },
];

export default function Resources() {
  return (
    <div className="min-h-screen">
      {/* Navigation */}
      <nav className="fixed top-0 left-0 right-0 z-50 bg-[#0F172A]/80 backdrop-blur-md border-b border-[#334155]">
        <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-3">
            <Image src="/logo-icon.svg" alt="Syslens" width={32} height={32} />
            <span className="text-xl font-semibold">Syslens</span>
          </Link>
          <div className="flex items-center gap-6">
            <Link href="/#features" className="text-[#94A3B8] hover:text-white transition-colors">Features</Link>
            <Link href="/#gallery" className="text-[#94A3B8] hover:text-white transition-colors">Gallery</Link>
            <Link href="/resources" className="text-white font-medium">Resources</Link>
            <a href="https://github.com/lhanks/syslens" className="text-[#94A3B8] hover:text-white transition-colors">GitHub</a>
            <Link
              href="/#download"
              className="px-4 py-2 bg-[#3B82F6] hover:bg-[#60A5FA] text-white rounded-lg transition-colors"
            >
              Download
            </Link>
          </div>
        </div>
      </nav>

      {/* Header */}
      <section className="pt-32 pb-12 px-6">
        <div className="max-w-4xl mx-auto text-center">
          <h1 className="text-4xl md:text-5xl font-bold mb-4">
            Resources & Links
          </h1>
          <p className="text-xl text-[#94A3B8] max-w-2xl mx-auto">
            Quick access to all Syslens project resources, repositories, and documentation.
          </p>
        </div>
      </section>

      {/* Resources Grid */}
      <section className="py-12 px-6">
        <div className="max-w-5xl mx-auto">
          {resources.map((section, sectionIndex) => (
            <div key={sectionIndex} className="mb-12">
              <h2 className="text-2xl font-bold mb-6 text-[#94A3B8]">{section.category}</h2>
              <div className="grid md:grid-cols-2 gap-4">
                {section.items.map((item, itemIndex) => (
                  <a
                    key={itemIndex}
                    href={item.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="card p-6 flex items-start gap-4 hover:border-[#3B82F6]/50 transition-all group"
                  >
                    <div className="w-12 h-12 rounded-xl bg-[#3B82F6]/20 flex items-center justify-center text-[#3B82F6] group-hover:bg-[#3B82F6]/30 transition-colors flex-shrink-0">
                      {item.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="text-lg font-semibold mb-1 group-hover:text-[#3B82F6] transition-colors">
                        {item.title}
                        <svg className="w-4 h-4 inline-block ml-2 opacity-0 group-hover:opacity-100 transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                        </svg>
                      </h3>
                      <p className="text-[#94A3B8] text-sm">{item.description}</p>
                      <p className="text-[#64748B] text-xs mt-2 truncate">{item.url}</p>
                    </div>
                  </a>
                ))}
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Back to Home */}
      <section className="py-12 px-6">
        <div className="max-w-4xl mx-auto text-center">
          <Link
            href="/"
            className="inline-flex items-center gap-2 px-6 py-3 border border-[#334155] hover:border-[#3B82F6] text-white rounded-xl font-semibold transition-colors"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
            Back to Home
          </Link>
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
            <Link href="/resources" className="text-[#94A3B8] hover:text-white transition-colors">
              Resources
            </Link>
          </div>
        </div>
      </footer>
    </div>
  );
}
