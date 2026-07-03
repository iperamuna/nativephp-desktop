<?php

namespace Native\Desktop\Drivers\Tauri\Commands;

use ZipArchive;
use Illuminate\Support\Facades\File;

use Illuminate\Console\Command;
use Native\Desktop\Builder\Builder;
use Native\Desktop\Drivers\Tauri\Traits\Developer;
use Native\Desktop\Drivers\Tauri\Traits\Installer;
use Symfony\Component\Console\Attribute\AsCommand;

use function Laravel\Prompts\intro;
use function Laravel\Prompts\note;

#[AsCommand(
    name: 'native:run',
    description: 'Start the NativePHP Tauri development server',
)]
class RunCommand extends Command
{
    use Developer;
    use Installer;

    protected $signature = 'native:run {--no-queue} {--no-focus} {--D|no-dependencies} {--installer=npm}';

    public function __construct(
        protected Builder $builder
    ) {
        parent::__construct();
    }

    public function handle(): void
    {
        intro('Starting NativePHP Tauri dev server…');

        note('Fetching latest dependencies…');

        if (! $this->option('no-dependencies')) {
            $this->installNPMDependencies(
                force: true,
                installer: $this->option('installer'),
                withoutInteraction: $this->option('no-interaction')
            );
        }

        note('Starting NativePHP Tauri app');

        // Copy certificates if needed for local SSL
        $this->builder->copyCertificateAuthority();

        $this->extractPhpBinary();

        $this->runDeveloper(
            installer: $this->option('installer'),
            skip_queue: $this->option('no-queue'),
            no_focus: $this->option('no-focus'),
            withoutInteraction: $this->option('no-interaction')
        );
    }

    protected function extractPhpBinary(): void
    {
        $os = strtolower(PHP_OS_FAMILY);
        if ($os === 'darwin') {
            $os = 'mac';
        }

        $arch = php_uname('m');
        if ($arch === 'aarch64') {
            $arch = 'arm64';
        }

        $phpVersion = config('nativephp.version', '8.4');
        
        $zipPath = base_path("vendor/nativephp/php-bin/bin/{$os}/{$arch}/php-{$phpVersion}.zip");
        
        if (! file_exists($zipPath)) {
            // Fallback to 8.4 if the config version doesn't exist in php-bin
            $phpVersion = '8.4';
            $zipPath = base_path("vendor/nativephp/php-bin/bin/{$os}/{$arch}/php-{$phpVersion}.zip");
        }
        
        $targetName = match (true) {
            $os === 'mac' && $arch === 'arm64' => 'aarch64-apple-darwin',
            $os === 'mac' && $arch === 'x86_64' => 'x86_64-apple-darwin',
            $os === 'linux' && $arch === 'x86_64' => 'x86_64-unknown-linux-gnu',
            $os === 'windows' && $arch === 'x86_64' => 'x86_64-pc-windows-msvc',
            default => 'x86_64-unknown-linux-gnu',
        };

        $binDir = base_path('nativephp/tauri/src-tauri/bin');
        File::ensureDirectoryExists($binDir);
        
        $targetBinary = $binDir . '/php-' . $targetName;
        if ($os === 'windows') {
            $targetBinary .= '.exe';
        }

        if (! file_exists($targetBinary) || filesize($targetBinary) === 0) {
            note('Extracting PHP binary for Tauri...');
            $zip = new ZipArchive();
            if ($zip->open($zipPath) === true) {
                $binaryName = $os === 'windows' ? 'php.exe' : 'php';
                $content = $zip->getFromName($binaryName);
                if ($content !== false) {
                    file_put_contents($targetBinary, $content);
                    chmod($targetBinary, 0755);
                } else {
                    $this->error('Failed to find php inside the zip archive.');
                }
                $zip->close();
            } else {
                $this->error('Failed to open PHP binary zip: ' . $zipPath);
            }
        }
    }
}
