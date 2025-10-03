import React from 'react'
import { Link } from 'react-router-dom'

export default function Home() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 to-gray-800">
      {/* Navigation */}
      <nav className="container mx-auto px-6 py-4">
        <div className="flex justify-between items-center">
          <div className="text-2xl font-bold text-blue-400">🔐 Vault</div>
          <div className="space-x-6">
            <Link to="/docs" className="hover:text-blue-400 transition-colors">Docs</Link>
            <Link to="/demo" className="hover:text-blue-400 transition-colors">Demo</Link>
            <a href="https://github.com/vault/vault" className="hover:text-blue-400 transition-colors">GitHub</a>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="container mx-auto px-6 py-20 text-center">
        <h1 className="text-6xl font-bold mb-6 bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
          Local-First Password Manager
        </h1>
        <p className="text-xl text-gray-300 mb-8 max-w-3xl mx-auto">
          Secure, encrypted, multi-tenant password management with optional cloud sync. 
          Built with Rust for maximum security and performance.
        </p>
        
        <div className="space-x-4 mb-12">
          <button className="btn-primary text-lg px-8 py-3">
            Download for Linux
          </button>
          <button className="btn-secondary text-lg px-8 py-3">
            View on GitHub
          </button>
        </div>

        {/* Installation */}
        <div className="max-w-2xl mx-auto">
          <h3 className="text-lg font-semibold mb-4">Quick Install</h3>
          <div className="code-block text-left">
            curl -sSL https://releases.vault.dev/install.sh | sh
          </div>
        </div>
      </section>

      {/* Features */}
      <section className="container mx-auto px-6 py-20">
        <h2 className="text-4xl font-bold text-center mb-16">Why Vault?</h2>
        
        <div className="grid md:grid-cols-3 gap-8">
          <div className="bg-gray-800 p-8 rounded-lg">
            <div className="text-3xl mb-4">🔒</div>
            <h3 className="text-xl font-semibold mb-4">Zero-Knowledge Encryption</h3>
            <p className="text-gray-300">
              AES-256-GCM and ChaCha20-Poly1305 encryption. Your secrets are encrypted 
              client-side before any cloud storage.
            </p>
          </div>
          
          <div className="bg-gray-800 p-8 rounded-lg">
            <div className="text-3xl mb-4">🏠</div>
            <h3 className="text-xl font-semibold mb-4">Local-First</h3>
            <p className="text-gray-300">
              Works completely offline. Cloud sync is optional and always encrypted.
              Your data stays on your device by default.
            </p>
          </div>
          
          <div className="bg-gray-800 p-8 rounded-lg">
            <div className="text-3xl mb-4">🏢</div>
            <h3 className="text-xl font-semibold mb-4">Multi-Tenant</h3>
            <p className="text-gray-300">
              Organizations, projects, and role-based access control. 
              Perfect for teams and enterprise use.
            </p>
          </div>
        </div>
      </section>

      {/* CLI Demo */}
      <section className="container mx-auto px-6 py-20">
        <h2 className="text-4xl font-bold text-center mb-16">Beautiful CLI</h2>
        
        <div className="max-w-4xl mx-auto">
          <div className="bg-gray-900 rounded-lg p-6 border border-gray-700">
            <div className="code-block">
              <div className="text-gray-500"># Initialize vault for your organization</div>
              <div>$ vault init --tenant acme-corp --admin alice@acme.com</div>
              <div className="text-green-400">✓ Vault initialized successfully</div>
              <br />
              <div className="text-gray-500"># Store a secret</div>
              <div>$ vault put github-token --namespace development</div>
              <div className="text-blue-400">Enter secret value: ••••••••••••••••</div>
              <div className="text-green-400">✓ Secret stored: development/github-token</div>
              <br />
              <div className="text-gray-500"># Sync across devices</div>
              <div>$ vault sync push</div>
              <div className="text-green-400">✓ 3 secrets synced to cloud</div>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="container mx-auto px-6 py-8 border-t border-gray-700">
        <div className="text-center text-gray-400">
          <p>&copy; 2024 Vault. Open source under MIT license.</p>
        </div>
      </footer>
    </div>
  )
}