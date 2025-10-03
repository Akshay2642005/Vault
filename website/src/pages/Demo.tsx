import React from 'react'
import { Link } from 'react-router-dom'

export default function Demo() {
  return (
    <div className="min-h-screen bg-gray-900">
      <nav className="container mx-auto px-6 py-4 border-b border-gray-700">
        <Link to="/" className="text-2xl font-bold text-blue-400">🔐 Vault</Link>
      </nav>

      <div className="container mx-auto px-6 py-8">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-4xl font-bold mb-8">Interactive Demo</h1>
          
          <div className="bg-gray-800 rounded-lg p-6 mb-8">
            <h2 className="text-xl font-semibold mb-4">Try Vault Commands</h2>
            <p className="text-gray-300 mb-4">
              Experience the Vault CLI in your browser. All operations are simulated - no real secrets are stored.
            </p>
            
            <div className="bg-gray-900 rounded p-4 font-mono text-sm">
              <div className="text-green-400">vault@demo:~$ </div>
              <div className="text-gray-300 mt-2">
                Try these commands:<br/>
                • <span className="text-blue-400">vault init --tenant demo --admin demo@example.com</span><br/>
                • <span className="text-blue-400">vault put github-token --namespace dev</span><br/>
                • <span className="text-blue-400">vault list --namespace dev</span><br/>
                • <span className="text-blue-400">vault get github-token --namespace dev</span>
              </div>
            </div>
          </div>

          <div className="grid md:grid-cols-2 gap-8">
            <div className="bg-gray-800 rounded-lg p-6">
              <h3 className="text-lg font-semibold mb-4">🔐 Encryption Demo</h3>
              <p className="text-gray-300 mb-4">
                See how your secrets are encrypted before storage:
              </p>
              <div className="bg-gray-900 rounded p-3 text-sm">
                <div className="text-gray-400">Plaintext:</div>
                <div className="text-white">my-secret-password</div>
                <div className="text-gray-400 mt-2">Encrypted (AES-256-GCM):</div>
                <div className="text-green-400 break-all">
                  7a8f9e2d1c4b5a6e9f8d7c6b5a4e3d2c1b0a9f8e7d6c5b4a3e2d1c0b9a8f7e6d
                </div>
              </div>
            </div>

            <div className="bg-gray-800 rounded-lg p-6">
              <h3 className="text-lg font-semibold mb-4">☁️ Sync Demo</h3>
              <p className="text-gray-300 mb-4">
                Multi-device synchronization workflow:
              </p>
              <div className="space-y-2 text-sm">
                <div className="flex items-center">
                  <span className="text-green-400 mr-2">✓</span>
                  <span>Device A: Create secret</span>
                </div>
                <div className="flex items-center">
                  <span className="text-green-400 mr-2">✓</span>
                  <span>Device A: Push to cloud (encrypted)</span>
                </div>
                <div className="flex items-center">
                  <span className="text-green-400 mr-2">✓</span>
                  <span>Device B: Pull from cloud</span>
                </div>
                <div className="flex items-center">
                  <span className="text-green-400 mr-2">✓</span>
                  <span>Device B: Decrypt locally</span>
                </div>
              </div>
            </div>
          </div>

          <div className="mt-8 text-center">
            <Link to="/docs" className="btn-primary">
              Read Full Documentation
            </Link>
          </div>
        </div>
      </div>
    </div>
  )
}