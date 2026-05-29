import 'package:file_picker/file_picker.dart';
import 'package:permission_handler/permission_handler.dart';

Future<void> pickAIMFFile() async {
  final result = await FilePicker.platform.pickFiles(
    type: FileType.custom,
    allowedExtensions: ['avid', 'aimg', 'aaud'],
    allowMultiple: false,
  );
  
  if (result != null) {
    final file = File(result.files.single.path!);
    final bytes = await file.readAsBytes();
    final mimeType = AIMFMimeTypes.detectFromBytes(bytes);
    
    print('Opened ${result.files.single.name} as $mimeType');
  }
}

// Share file with other apps
Future<void> shareAIMFFile(String path) async {
  final file = File(path);
  final mimeType = AIMFMimeTypes.extensionToMime[path.split('.').last];
  
  await Share.shareXFiles(
    [XFile(path, mimeType: mimeType)],
    text: 'Sharing AIMF file',
  );
}