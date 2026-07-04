<?php

use Illuminate\Support\Facades\Http;
use Native\Desktop\Facades\Window;
use Native\Desktop\Facades\Notification;
use Native\Desktop\Facades\Clipboard;
use Native\Desktop\Facades\Dialog;
use Native\Desktop\Facades\GlobalShortcut;

beforeEach(function () {
    // Fake the HTTP requests that the NativePHP Client makes to the Tauri Bridge
    Http::fake([
        'http://127.0.0.1:8100/_native/api/*' => Http::response(['success' => true], 200),
    ]);

    // Mock environment variables injected by Tauri Bridge
    config(['nativephp-internal.api_url' => 'http://127.0.0.1:8100/_native/api/']);
    config(['nativephp.running' => true]);
});

it('can dispatch window open commands to the tauri bridge', function () {
    Window::open('main_window');

    Http::assertSent(function ($request) {
        return $request->url() == 'http://127.0.0.1:8100/_native/api/window/open' &&
               $request['id'] == 'main_window';
    });
});

it('can dispatch notification commands to the tauri bridge', function () {
    Notification::title('Hello')
        ->message('World')
        ->show();

    Http::assertSent(function ($request) {
        return $request->url() == 'http://127.0.0.1:8100/_native/api/notification' &&
               $request['title'] == 'Hello' &&
               $request['body'] == 'World'; // Or whatever NativePHP maps message to
    });
});

it('can dispatch clipboard write commands to the tauri bridge', function () {
    Clipboard::text('Secret Password');

    Http::assertSent(function ($request) {
        return $request->url() == 'http://127.0.0.1:8100/_native/api/clipboard/text' &&
               $request['text'] == 'Secret Password';
    });
});

it('verifies tauri bridge rust compilation successfully', function () {
    $result = exec('cd ' . __DIR__ . '/../resources/tauri/src-tauri && cargo test 2>&1', $output, $status);
    
    expect($status)->toBe(0);
});
