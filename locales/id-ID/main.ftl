# $filename (String) - Nama file sama! akan menyebabkan overwrite file pada file tersebut.
docx-filler-fail-overwrite =
    File \" { $filename } \" tersedia!
    saya tidak akan mengubah file yang sudah tersedia!
docx-filler-fail-load = gagal menganalisa dan membaca konten DOCX template!

valid-no-tokens = tidak ada token yang tersedia!
# $token (String) - The token that user tried to use multiple times in replacements.
valid-token-duplicity = Token { $token } dapat digunakan hanya sekali!
valid-missing-input = Tidak dapat menemnukan value input!
# $tokens (String) - Jumlah Token pada input.
# $values (String) - Jumlah value pada input.
valid-count-mismatch =
    Jumlah token tidak sama dengan jumlah value pada on input token!
    tokens: { $tokens }
    values: { $values }
# $line (String) - Number of input line where the problem is.
# $details (String) - Problem details.
valid-line-mismatch =
    Terjadi Kesalahan pada baris: { $line } didalam input:
    { $details }
# $filename (String) - Nama Hasil File output tidak memenuhi kriteria .docx ekstensi.
valid-no-docx-suffix =
    Nama Output File hasil di akhiri dengan ekstensi .docx!
    You have: "{ $filename }".
# $filename (String) - Nama dari beberapa File output harus mengikuti peraturan penamaan!.
valid-same-output-filename = Lebih dari satu input line harus memiliki nama file yang sama "{ $filename }"!

#
ui-docx-app-title = Piagam Dispendik Filler
ui-docx-load-failed = Gagal Memuat file Docx!
ui-docx-generated = Berhasil Mengenerate / Membuat File Docx dari inputan.
ui-docx-success =  Sukses!
ui-docx-failure = Waduh...
ui-docx-no-template = Anda belum memberikan input file template yang akan di isi!
ui-docx-fail-font = Terjadi Kegagalam saat menyetel font default!
ui-docx-fail-init = Terjadi Kegagalan saat inisialisasi Native Windows GUI!
ui-docx-fail-build = Terjadi Kegagalan pada interface builder!
#
ui-template-label = Template saat ini yang termuat:
ui-template-dialog = Buka File Template
ui-template-button = Muat Docx Baru
ui-template-default-folder-fail = Terjadi Kesalahan saat menyetel folder kerja!
#
ui-data-dialog = Buka File Data Input
ui-data-label = Data saat ini yang termuat:
ui-data-button = Muat Data Baru
ui-data-default-folder-fail = Terjadi Kesalahan saat menyetel folder kerja!
#
ui-tokens-label = Tokens (placeholders) ditemukan pada file:
ui-tokens-failed-sep-create = Gagal Membuat Karakter Pemisah!
ui-tokens-failed-sep-add = Terjadi Kesalahan saat menambahkan Karakter Pemisah pada Layout!
ui-tokens-failed-tok-create = Terjadi Kesalahan saat membuat token!
ui-tokens-failed-tok-add = Terjadi Kesalahan saat menambahkan token pada Layout!
#
ui-values-label = Isi Setiap Value sesuai dengan Token (Satu Line per Halaman):
#
ui-output-label = Pola Nama File Output:
ui-output-button = Hasilkan Docx File
ui-options-sep-label = Karakter Pemisah:

lang-not-found = Tidak dapat mengubah bahasa pada bahasa yang diminta!
