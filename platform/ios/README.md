# AIMF iOS Integration Guide

## Quick Setup

### 1. Add UTI Definition to Your Xcode Project

Copy `AIMF.uttype` to your Xcode project:

```bash
cp platform/ios/AIMF.uttype YourApp/Resources/

2. Configure Info.plist

Merge the contents of Info.plist (above) into your app's Info.plist.

Key sections to add:
xml

<!-- Import UTIs -->
<key>UTImportedTypeDeclarations</key>
<array>...</array>

<!-- Document Types -->
<key>CFBundleDocumentTypes</key>
<array>...</array>

<!-- Exported UTIs -->
<key>UTExportedTypeDeclarations</key>
<array>...</array>

3. Register in Your App Delegate
swift

// AppDelegate.swift
import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    
    func application(_ app: UIApplication, open url: URL, 
                     options: [UIApplication.OpenURLOptionsKey : Any] = [:]) -> Bool {
        
        // Handle AIMF file opening
        if url.pathExtension.lowercased() == "avid" ||
           url.pathExtension.lowercased() == "aimg" ||
           url.pathExtension.lowercased() == "aaud" {
            
            // Navigate to your AIMF viewer
            NotificationCenter.default.post(
                name: NSNotification.Name("OpenAIMFFile"),
                object: url
            )
            return true
        }
        return false
    }
}

4. Handle File in Your View Controller
swift

// ViewController.swift
import UIKit
import UniformTypeIdentifiers

class ViewController: UIViewController {
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        // Listen for file open events
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleAIMFFile(_:)),
            name: NSNotification.Name("OpenAIMFFile"),
            object: nil
        )
    }
    
    @objc func handleAIMFFile(_ notification: Notification) {
        guard let url = notification.object as? URL else { return }
        
        // Verify the file
        verifyAIMFFile(at: url)
    }
    
    // File picker for opening AIMF files
    @IBAction func openAIMFFile(_ sender: Any) {
        let supportedTypes: [UTType] = [
            UTType("com.aimf.video")!,
            UTType("com.aimf.image")!,
            UTType("com.aimf.audio")!,
            UTType("com.aimf.generic")!
        ]
        
        let documentPicker = UIDocumentPickerViewController(
            forOpeningContentTypes: supportedTypes,
            asCopy: true
        )
        documentPicker.delegate = self
        documentPicker.allowsMultipleSelection = false
        present(documentPicker, animated: true)
    }
    
    func verifyAIMFFile(at url: URL) {
        // Call AIMF CLI (if embedded) or use Rust FFI
        let task = Process()
        task.executableURL = Bundle.main.url(forResource: "aimf", withExtension: nil)
        task.arguments = ["verify", url.path]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8)
            
            if task.terminationStatus == 0 {
                showAlert(title: "✅ Valid AIMF File", message: output)
            } else {
                showAlert(title: "❌ Invalid AIMF File", message: output)
            }
        } catch {
            showAlert(title: "Error", message: error.localizedDescription)
        }
    }
    
    func showAlert(title: String, message: String?) {
        let alert = UIAlertController(title: title, message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "OK", style: .default))
        present(alert, animated: true)
    }
}

extension ViewController: UIDocumentPickerDelegate {
    func documentPicker(_ controller: UIDocumentPickerViewController, 
                       didPickDocumentsAt urls: [URL]) {
        guard let url = urls.first else { return }
        verifyAIMFFile(at: url)
    }
}

5. Swift Package Manager Integration

Package.swift:
swift

// swift-tools-version:5.5
import PackageDescription

let package = Package(
    name: "AIMFKit",
    platforms: [
        .iOS(.v14),
        .macOS(.v11)
    ],
    products: [
        .library(
            name: "AIMFKit",
            targets: ["AIMFKit"]),
    ],
    targets: [
        .target(
            name: "AIMFKit",
            dependencies: [],
            path: "Sources",
            resources: [
                .copy("Resources/AIMF.uttype")
            ]
        )
    ]
)

6. SwiftUI Integration
swift

import SwiftUI
import UniformTypeIdentifiers

struct AIMFDocumentPicker: UIViewControllerRepresentable {
    var onPick: (URL) -> Void
    
    func makeUIViewController(context: Context) -> UIDocumentPickerViewController {
        let types = [
            UTType("com.aimf.video")!,
            UTType("com.aimf.image")!,
            UTType("com.aimf.audio")!
        ]
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: types, asCopy: true)
        picker.delegate = context.coordinator
        return picker
    }
    
    func updateUIViewController(_ uiViewController: UIDocumentPickerViewController, context: Context) {}
    
    func makeCoordinator() -> Coordinator {
        Coordinator(onPick: onPick)
    }
    
    class Coordinator: NSObject, UIDocumentPickerDelegate {
        var onPick: (URL) -> Void
        
        init(onPick: @escaping (URL) -> Void) {
            self.onPick = onPick
        }
        
        func documentPicker(_ controller: UIDocumentPickerViewController, 
                           didPickDocumentsAt urls: [URL]) {
            guard let url = urls.first else { return }
            onPick(url)
        }
    }
}

// Usage in SwiftUI
struct ContentView: View {
    @State private var selectedFileURL: URL?
    
    var body: some View {
        VStack {
            if let url = selectedFileURL {
                Text("Selected: \(url.lastPathComponent)")
            }
            
            Button("Open AIMF File") {
                // Show document picker
            }
        }
    }
}

Testing on Device

    AirDrop an AIMF file to your iOS device

    Tap the file — your app should appear in the share sheet

    Long press on the file → "Open with..." → your app

Troubleshooting
"Unknown file type" on iOS

Solution:
bash

# Re-register UTIs
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f YourApp.app

App not appearing in share sheet

Check Info.plist has:
xml

<key>CFBundleDocumentTypes</key>
<array>...</array>

Files not opening after app install

Reboot device — iOS caches UTI registrations.
Resources

    Apple Uniform Type Identifiers

    Document Picker Documentation