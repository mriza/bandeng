# Bandeng - Easy Mikrotik Hotspot IP Binding Management

Aplikasi desktop modern untuk mengelola IP binding pada MikroTik Hotspot dengan antarmuka yang intuitif dan ringan.

## 🚀 Fitur

- **Otentikasi Aman**: Login ke router MikroTik via API.
- **Manajemen IP Binding**: Lihat, tambah, dan hapus binding dengan mudah.
- **Status Visual**: Dilengkapi dengan badge status (`Bypassed`, `Blocked`, `Regular`).
- **Offline Ready**: Semua aset (CSS & Ikon) berjalan luring tanpa butuh internet.
- **Multi-Platform**: Tersedia untuk Windows dan Linux.

## 🛠️ Persyaratan

- Router MikroTik dengan RouterOS v6 atau v7.
- Layanan API MikroTik diaktifkan.

## ⚙️ Konfigurasi MikroTik API

Agar aplikasi dapat terhubung, jalankan perintah berikut di terminal MikroTik Anda:

### 1. Aktifkan Layanan API
```bash
/ip service enable api
```

### 2. Siapkan User & Group (Direkomendasikan)
Buat group dengan izin terbatas untuk keamanan maksimal:
```bash
/user group add name=bandeng-group policy=read,write,api,!local,!telnet,!ssh,!ftp,!reboot,!policy,!test,!winbox,!password,!web,!sniff,!sensitive,!romon,!rest-api
/user add name=bandeng-user group=bandeng-group password=PASSWORD_ANDA
```

## 📥 Instalasi

### 🔹 Cara Cepat (Download Binary)
1. Buka halaman **Releases** di repositori ini.
2. Unduh file sesuai sistem operasi Anda:
   - **Windows**: Download `Bandeng-amd64.exe`.
   - **Linux**: Download `Bandeng-linux-amd64`, berikan izin eksekusi (`chmod +x Bandeng-linux-amd64`), lalu jalankan.

### 🔹 Build dari Source
Jika Anda ingin melakukan build sendiri:
1. Pastikan sudah terinstall **Go 1.21+** dan **Wails v2**.
2. Clone repositori ini.
3. Masuk ke folder proyek dan jalankan:
   ```bash
   wails build
   ```
4. Hasil build akan tersedia di folder `build/bin/`.

## 📖 Penggunaan

1. Jalankan aplikasi **Bandeng**.
2. Masukkan alamat IP Router, Username, dan Password API.
3. Klik **🔗 Connect**.
4. Gunakan tombol **➕ Add Binding** untuk menambah data baru atau ikon 🗑️ untuk menghapus.
5. Klik **⛓️‍💥 Disconnect** untuk keluar.

## ❓ Troubleshooting

- **Connection Refused**: Pastikan port API (8728) tidak diblokir oleh firewall router.
- **Login Failed**: Periksa kembali username, password, dan pastikan user tersebut masuk ke group yang memiliki izin `api`.
- **No Such Command**: Pastikan RouterOS Anda mendukung fitur Hotspot.