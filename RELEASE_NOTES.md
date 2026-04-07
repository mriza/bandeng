# Release Notes - Bandeng v1.1.0

## 📦 Apa yang Baru?

Versi **1.1.0** merupakan pembaruan mayor yang memfokuskan pada **Keamanan (Security)** dan **Efisiensi Operasional**. Kami mendengarkan masukan pengguna mengenai pengisian tabel IP Binding yang memakan waktu dan kebutuhan akan koneksi terenkripsi.

### 🛡️ Keamanan (SSL/TLS Core)
Kini Anda dapat menghubungkan aplikasi ke router melalui protokol **API-SSL**. Jika router Anda menggunakan sertifikat yang ditandatangani sendiri (*self-signed*), Bandeng akan membantu Anda:
- Menampilkan pesan peringatan saat mendeteksi sertifikat untrusted.
- Menyediakan detail sertifikat untuk diinspeksi.
- Menampilkan **SHA256 Fingerprint** agar Anda bisa memastikan tidak ada serangan penyadapan.

### ⚡ Sinkronisasi Pintar (Sync ARP)
Tidak perlu lagi mengetikkan alamat IP satu per satu. Dengan fitur sinkronisasi baru:
- **Sync All**: Klik satu tombol, dan biarkan aplikasi mencari semua pasangan MAC-to-IP dari tabel ARP MikroTik Anda.
- **Diferensiasi Ikon**: Ikon **Refresh** (memuat ulang daftar) dan **Sync** (pencarian ARP) kini dibedakan untuk pengalaman pengguna yang lebih baik.

### 🛠️ Perbaikan & Peningkatan
- **State Preservation**: Saat Anda mengupdate data di halaman 2, 3, atau seterusnya, aplikasi tidak akan lagi mereset tampilan ke halaman pertama.
- **Correct Data Binding**: Memperbaiki bug di mana kolom Alamat IP terkadang tetap kosong meskipun data di router sudah terisi.
- **Labeling**: Tombol-tombol utama kini dilengkapi label teks (Refresh, Sync All, Add Binding) untuk kemudahan navigasi.

---

## 🚀 Cara Update

1. Hapus versi binary lama.
2. Unduh versi **v1.1.0** sesuai OS Anda.
3. Pastikan layanan `api-ssl` aktif di router jika ingin menggunakan mode Secure Login.

---

**Tim Pengembang Bandeng**
*Modern, Easy, and Secure MikroTik Management.*
