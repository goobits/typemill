/**
 * TLS certificate utilities for CodeFlow Buddy
 * Provides helpers for generating and validating TLS certificates
 */

import { execSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { logger } from '../core/logger.js';

export interface CertificateGenerationOptions {
  outputDir: string;
  keyFile?: string;
  certFile?: string;
  days?: number;
  country?: string;
  state?: string;
  city?: string;
  organization?: string;
  organizationalUnit?: string;
  commonName?: string;
  subjectAltNames?: string[];
}

export class TLSCertificateManager {
  private static readonly DEFAULT_OPTIONS: Required<Omit<CertificateGenerationOptions, 'outputDir' | 'subjectAltNames'>> & { subjectAltNames: string[] } = {
    keyFile: 'server.key',
    certFile: 'server.crt',
    days: 365,
    country: 'US',
    state: 'CA',
    city: 'San Francisco',
    organization: 'CodeFlow Buddy',
    organizationalUnit: 'Development',
    commonName: 'localhost',
    subjectAltNames: ['DNS:localhost', 'IP:127.0.0.1', 'IP:::1']
  };

  /**
   * Generate a self-signed certificate for development use
   */
  static generateSelfSignedCertificate(options: CertificateGenerationOptions): { keyPath: string; certPath: string } {
    const opts = { ...this.DEFAULT_OPTIONS, ...options };

    // Ensure output directory exists
    if (!existsSync(opts.outputDir)) {
      mkdirSync(opts.outputDir, { recursive: true });
    }

    const keyPath = join(opts.outputDir, opts.keyFile);
    const certPath = join(opts.outputDir, opts.certFile);

    try {
      // Generate private key
      logger.info('Generating TLS private key', {
        component: 'TLSCertificateManager',
        keyPath
      });

      execSync(`openssl genrsa -out "${keyPath}" 2048`, { stdio: 'pipe' });

      // Create certificate signing request configuration
      const subject = `/C=${opts.country}/ST=${opts.state}/L=${opts.city}/O=${opts.organization}/OU=${opts.organizationalUnit}/CN=${opts.commonName}`;

      const sanExtension = opts.subjectAltNames.length > 0
        ? `subjectAltName=${opts.subjectAltNames.join(',')}`
        : '';

      logger.info('Generating self-signed certificate', {
        component: 'TLSCertificateManager',
        certPath,
        subject,
        days: opts.days,
        subjectAltNames: opts.subjectAltNames
      });

      // Generate self-signed certificate
      const opensslCmd = [
        'openssl req',
        '-new',
        '-x509',
        `-key "${keyPath}"`,
        `-out "${certPath}"`,
        `-days ${opts.days}`,
        `-subj "${subject}"`,
        ...(sanExtension ? ['-extensions SAN', `-config <(echo "[SAN]\\n${sanExtension}")`] : [])
      ].join(' ');

      if (sanExtension) {
        // Use bash to handle process substitution for SAN
        execSync(`bash -c '${opensslCmd}'`, { stdio: 'pipe' });
      } else {
        execSync(opensslCmd, { stdio: 'pipe' });
      }

      logger.info('TLS certificate generated successfully', {
        component: 'TLSCertificateManager',
        keyPath,
        certPath,
        expiryDays: opts.days
      });

      return { keyPath, certPath };

    } catch (error) {
      logger.error('Failed to generate TLS certificate', error as Error, {
        component: 'TLSCertificateManager',
        outputDir: opts.outputDir
      });

      throw new Error(`Certificate generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Validate that certificate files exist and are readable
   */
  static validateCertificateFiles(keyPath: string, certPath: string, caPath?: string): boolean {
    try {
      if (!existsSync(keyPath)) {
        logger.error('TLS private key file not found', new Error('File not found'), {
          component: 'TLSCertificateManager',
          keyPath
        });
        return false;
      }

      if (!existsSync(certPath)) {
        logger.error('TLS certificate file not found', new Error('File not found'), {
          component: 'TLSCertificateManager',
          certPath
        });
        return false;
      }

      if (caPath && !existsSync(caPath)) {
        logger.error('TLS CA certificate file not found', new Error('File not found'), {
          component: 'TLSCertificateManager',
          caPath
        });
        return false;
      }

      logger.debug('TLS certificate files validated', {
        component: 'TLSCertificateManager',
        keyPath,
        certPath,
        caPath
      });

      return true;

    } catch (error) {
      logger.error('TLS certificate validation failed', error as Error, {
        component: 'TLSCertificateManager',
        keyPath,
        certPath,
        caPath
      });
      return false;
    }
  }

  /**
   * Get certificate information (requires openssl command)
   */
  static getCertificateInfo(certPath: string): {
    subject: string;
    issuer: string;
    notBefore: string;
    notAfter: string;
    fingerprint: string;
  } | null {
    try {
      const subject = execSync(`openssl x509 -in "${certPath}" -noout -subject`, { encoding: 'utf8' }).trim();
      const issuer = execSync(`openssl x509 -in "${certPath}" -noout -issuer`, { encoding: 'utf8' }).trim();
      const notBefore = execSync(`openssl x509 -in "${certPath}" -noout -startdate`, { encoding: 'utf8' }).trim();
      const notAfter = execSync(`openssl x509 -in "${certPath}" -noout -enddate`, { encoding: 'utf8' }).trim();
      const fingerprint = execSync(`openssl x509 -in "${certPath}" -noout -fingerprint -sha256`, { encoding: 'utf8' }).trim();

      return {
        subject: subject.replace('subject=', ''),
        issuer: issuer.replace('issuer=', ''),
        notBefore: notBefore.replace('notBefore=', ''),
        notAfter: notAfter.replace('notAfter=', ''),
        fingerprint: fingerprint.replace('SHA256 Fingerprint=', '')
      };

    } catch (error) {
      logger.error('Failed to get certificate information', error as Error, {
        component: 'TLSCertificateManager',
        certPath
      });
      return null;
    }
  }

  /**
   * Check if OpenSSL is available on the system
   */
  static isOpenSSLAvailable(): boolean {
    try {
      execSync('openssl version', { stdio: 'pipe' });
      return true;
    } catch {
      return false;
    }
  }
}