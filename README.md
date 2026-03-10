# Bandeng - Easy Mikrotik Hotspot IP Binding Management

Aplikasi desktop untuk mengelola IP binding pada Mikrotik Hotspot menggunakan Golang dan Wails.

## Fitur

- Otentikasi ke router Mikrotik
- Melihat daftar IP binding hotspot
- Menambah IP binding baru
- Menghapus IP binding
- Melihat informasi sistem router

## Persyaratan

- Go 1.21+
- Wails v2
- Router Mikrotik dengan API diaktifkan

## Instalasi

1. Clone atau download proyek ini.
2. Jalankan `go mod tidy` untuk mengunduh dependensi.
3. Build aplikasi: `wails build`

## Penggunaan

1. Jalankan aplikasi.
2. Masukkan alamat router, username, dan password.
3. Klik Connect.
4. Kelola IP bindings melalui UI.

## Troubleshooting

- Pastikan API Mikrotik diaktifkan di router.
- Periksa koneksi jaringan ke router.
- Username dan password harus memiliki hak akses yang cukup.