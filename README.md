# Bandeng - Easy Mikrotik Hotspot IP Binding Management

Aplikasi desktop modern untuk mengelola IP binding pada MikroTik Hotspot dengan antarmuka yang intuitif, ringan, dan aman.

## 🚀 Fitur Utama

- **Otentikasi Ganda**: Mendukung protokol **API (8728)** standar dan **API-SSL (8729)** yang terenkripsi.
- **Certificate Inspection**: Fitur keamanan untuk memeriksa sidik jari (fingerprint) SSL pada koneksi API-SSL, mencegah serangan Man-in-the-Middle.
- **Sinkronisasi IP dari ARP**: Secara otomatis mencari Alamat IP aktif dari Tabel ARP berdasarkan MAC Address.
    - **Sync Individual**: Sinkronisasi per baris data.
    - **Bulk Sync (Sync All)**: Memperbarui semua binding sekaligus dalam satu klik.
- **Manajemen IP Binding**: Lihat, tambah, dan hapus binding dengan mudah.
- **Dashboard Responsif**: Dilengkapi pagination dan status visual (`Bypassed`, `Blocked`, `Regular`).
- **Offline Ready**: Antarmuka ringan (dibangun secara native dengan Slint) tanpa overhead browser engine (WebView).
- **Multi-Platform**: Tersedia untuk Windows, macOS, dan Linux.

## ⚙️ Persyaratan & Konfigurasi

### 1. Persyaratan
- Router MikroTik dengan RouterOS v6 atau v7.
- Layanan API atau API-SSL diaktifkan.

### 2. Aktifkan Layanan API
Jalankan perintah berikut di terminal MikroTik:
```bash
# Untuk API standar
/ip service enable api

# Untuk API-SSL (Direkomendasikan)
/ip service enable api-ssl
```

### 3. Keamanan User (Direkomendasikan)
Agar aplikasi Bandeng dapat beroperasi dengan aman **khusus untuk mengatur IP Binding pada Hotspot**, disarankan untuk membuat grup dan user Mikrotik dengan izin terbatas:
```bash
/user group add name=bandeng-group policy=read,write,api,!local,!telnet,!ssh,!ftp,!reboot,!policy,!test,!winbox,!password,!web,!sniff,!sensitive,!romon,!rest-api
/user add name=bandeng-user group=bandeng-group password=PASSWORD_ANDA
```

## 📖 Cara Penggunaan

1. **Login**:
    - Masukkan alamat IP Router (contoh: `192.168.88.1`).
    - Masukkan Username dan Password.
    - Centang **Secure Login (SSL)** jika ingin menggunakan koneksi terenkripsi (Port 8729).
2. **Untrusted Certificate**:
    - Jika muncul peringatan sertifikat, periksa **SHA256 Fingerprint** yang tampil.
    - Cocokkan dengan fingerprint di Winbox (`/certificate print`).
    - Klik **Accept & Connect** jika sesuai.
3. **Dashboard**:
    - Klik tombol **Refresh** untuk menyegarkan daftar binding.
    - Klik tombol **🔍 Sync All** untuk otomatis mengisi alamat IP yang kosong dari tabel ARP.
    - Klik ikon **🔍 (Search Check)** pada baris tabel untuk sinkronisasi satu per satu.

## 📝 Changelog (v2.0.0)

- **[REWRITE]** Aplikasi sepenuhnya ditulis ulang menggunakan **Slint UI** (menggantikan Wails/Webview) untuk performa _native_, penggunaan RAM yang jauh lebih kecil, dan kecepatan maksimal.
- **[NEW]** Dukungan protokol API-SSL (TLS) dengan alur persetujuan Sertifikat SSL (TOFU).
- **[NEW]** Fitur "Sync All" untuk memindai ARP dan memperbarui semua IP kosong.
- **[NEW]** Fitur "Sync Selected" untuk menyinkronkan satu perangkat spesifik.
- **[NEW]** Penambahan Manajemen Perangkat (Tambah, Edit, Hapus IP Binding) yang dilengkapi Auto-Format MAC Address.
- **[IMPROVED]** Paginasi dan Sorting pada tabel, perbaikan batasan baris data RouterOS.

## 📥 Instalasi & Build

Unduh file binary terbaru di halaman **Releases** untuk OS Anda (Windows, Mac, Linux).

### Build dari Source
Aplikasi ini sekarang adalah aplikasi Rust native murni. Anda hanya memerlukan `cargo`:
```bash
# Pastikan Rust toolchain sudah terinstall
cargo build --release
```
File *binary* yang dihasilkan dapat ditemukan pada folder `target/release/bandeng-rs`.