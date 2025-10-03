import React from 'react'
import { Link } from 'react-router-dom'

export default function Docs() {
  return (
    <div className="min-h-screen bg-gray-900">
      <nav className="container mx-auto px-6 py-4 border-b border-gray-700">
        <Link to="/" className="text-2xl font-bold text-blue-400">🔐 Vault</Link>
      </nav>

      <div className="container mx-auto px-6 py-8">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-4xl font-bold mb-8">Documentation</h1>
          
          <div className="space-y-8">
            <section>
              <h2 className="text-2xl font-semibold mb-4">Installation</h2>
              <div className="code-block mb-4">
                # Install via script<br/>
                curl -sSL https://releases.vault.dev/install.sh | sh<br/><br/>
                # Or download binary directly<br/>
                wget https://github.com/vault/vault/releases/latest/download/vault-linux-amd64
              </div>
            </section>

            <section>
              <h2 className="text-2xl font-semibold mb-4">Quick Start</h2>
              <div className="code-block mb-4">
                # Initialize vault<br/>
                vault init --tenant my-org --admin admin@example.com<br/><br/>
                # Login<br/>
                vault login --tenant my-org<br/><br/>
                # Store a secret<br/>
                vault put api-key --namespace production<br/><br/>
                # Retrieve a secret<br/>
                vault get api-key --namespace production<br/><br/>
                # List secrets<br/>
                vault list --namespace production
              </div>
            </section>

            <section>
              <h2 className="text-2xl font-semibold mb-4">Cloud Sync Setup</h2>
              <div className="code-block mb-4">
                # Configure S3 backend<br/>
                vault config set cloud.backend S3<br/>
                vault config set cloud.bucket my-vault-bucket<br/>
                vault config set cloud.region us-east-1<br/><br/>
                # Push secrets to cloud<br/>
                vault sync push<br/><br/>
                # Pull secrets from cloud<br/>
                vault sync pull
              </div>
            </section>

            <section>
              <h2 className="text-2xl font-semibold mb-4">Security</h2>
              <ul className="list-disc list-inside space-y-2 text-gray-300">
                <li>All secrets encrypted with AES-256-GCM or ChaCha20-Poly1305</li>
                <li>Master keys derived using Argon2id with high memory cost</li>
                <li>Zero-knowledge architecture - server never sees plaintext</li>
                <li>Optional envelope encryption with AWS KMS/GCP KMS</li>
                <li>Memory-safe implementation with automatic secret zeroization</li>
              </ul>
            </section>

            <section>
              <h2 className="text-2xl font-semibold mb-4">Multi-Tenancy & RBAC</h2>
              <div className="code-block mb-4">
                # Add user to tenant<br/>
                vault roles add --tenant my-org --role writer --user bob@example.com<br/><br/>
                # Available roles: tenant_admin, owner, writer, reader, auditor<br/><br/>
                # Create namespace<br/>
                vault namespace create staging<br/><br/>
                # Set namespace permissions<br/>
                vault policy create staging-read --namespace staging --action read
              </div>
            </section>
          </div>
        </div>
      </div>
    </div>
  )
}