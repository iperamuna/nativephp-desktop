<?php

namespace Native\Desktop\Drivers\Tauri\Commands;

use Illuminate\Console\Command;
use Illuminate\Support\Facades\Process;
use Illuminate\Support\Str;
use Native\Desktop\Builder\Builder;
use Native\Desktop\Drivers\Tauri\TauriServiceProvider;
use Native\Desktop\Drivers\Tauri\Traits\OsAndArch;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Process\Process as SymfonyProcess;

use function Laravel\Prompts\intro;

#[AsCommand(
    name: 'native:build',
    description: 'Build the NativePHP application with Tauri for the specified operating system and architecture.',
)]
class BuildCommand extends Command
{
    use OsAndArch;

    protected $signature = 'native:build
        {os? : The operating system to build for (all, linux, mac, win)}
        {arch? : The Processor Architecture to build for (x64, arm64)}
        {--publish : to publish the app}';

    protected array $availableOs = ['win', 'linux', 'mac', 'all'];

    private string $buildCommand;

    private string $buildOS;

    public function __construct(
        protected Builder $builder
    ) {
        parent::__construct();
    }

    public function handle(): void
    {
        $this->buildOS = $this->selectOs($this->argument('os'));

        $this->buildCommand = 'build';
        if ($this->buildOS != 'all') {
            $arch = $this->selectArchitectureForOs($this->buildOS, $this->argument('arch'));

            $this->buildOS .= $arch != 'all' ? "-{$arch}" : '';
        }

        if ($this->option('publish')) {
            $this->buildCommand = 'publish';
        }

        if ($this->builder->hasBundled()) {
            $this->buildBundle();
        } else {
            $this->builder->warnUnsecureBuild();
            $this->buildUnsecure();
        }
    }

    private function buildBundle(): void
    {
        $this->builder->preProcess();
        
        $this->updateTauriDependencies();

        $this->newLine();
        intro('Copying Bundle to build directory...');
        $this->builder->copyBundleToBuildDirectory();

        $this->newLine();
        intro('Copying latest CA Certificate...');
        $this->builder->copyCertificateAuthority();

        $this->buildOrPublish();

        $this->builder->postProcess();
    }

    private function buildUnsecure(): void
    {
        $this->builder->preProcess();

        $this->updateTauriDependencies();

        $this->newLine();
        intro('Copying App to build directory...');
        $this->builder->copyToBuildDirectory();

        $this->newLine();
        intro('Copying latest CA Certificate...');
        $this->builder->copyCertificateAuthority();

        $this->newLine();
        intro('Cleaning .env file...');
        $this->builder->cleanEnvFile();

        $this->newLine();
        intro('Pruning vendor directory');
        $this->builder->pruneVendorDirectory();

        $this->buildOrPublish();

        $this->builder->postProcess();
    }

    protected function getEnvironmentVariables(): array
    {
        return [
            'APP_PATH' => $this->builder->sourcePath(),
            'APP_URL' => config('app.url'),
            'NATIVEPHP_BUILDING' => true,
            'NATIVEPHP_PHP_BINARY_VERSION' => PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION,
            'NATIVEPHP_PHP_BINARY_PATH' => $this->builder->phpBinaryPath(),
            'NATIVEPHP_TAURI_PATH' => TauriServiceProvider::tauriPath(),
            'NATIVEPHP_BUILD_PATH' => TauriServiceProvider::buildPath(),
            'NATIVEPHP_APP_NAME' => config('app.name'),
            'NATIVEPHP_APP_ID' => config('nativephp.app_id'),
            'NATIVEPHP_APP_VERSION' => config('nativephp.version'),
            'NATIVEPHP_APP_COPYRIGHT' => config('nativephp.copyright'),
            'NATIVEPHP_APP_FILENAME' => Str::slug(config('app.name')),
            'NATIVEPHP_APP_AUTHOR' => config('nativephp.author'),
        ];
    }

    private function updateTauriDependencies(): void
    {
        $this->newLine();
        intro('Updating Tauri dependencies...');
        Process::path(TauriServiceProvider::tauriPath())
            ->env($this->getEnvironmentVariables())
            ->forever()
            ->run('npm install', function (string $type, string $output) {
                echo $output;
            });
    }

    private function buildOrPublish(): void
    {
        $this->newLine();
        intro((($this->buildCommand == 'publish') ? 'Publishing' : 'Building')." for {$this->buildOS}");
        
        $target = $this->getTauriTarget($this->buildOS);
        
        $this->copyPhpSidecar($target);

        Process::path(TauriServiceProvider::tauriPath())
            ->env($this->getEnvironmentVariables())
            ->forever()
            ->tty(SymfonyProcess::isTtySupported() && ! $this->option('no-interaction'))
            ->run("npm run tauri build" . ($target ? " -- --target {$target}" : ""), function (string $type, string $output) {
                echo $output;
            });
    }

    private function getTauriTarget(string $osAndArch): ?string
    {
        // Convert to rust target triples if possible
        // Example: win-x64 -> x86_64-pc-windows-msvc
        // This is a naive implementation and might need expansion
        return match ($osAndArch) {
            'win-x64' => 'x86_64-pc-windows-msvc',
            'mac-x64' => 'x86_64-apple-darwin',
            'mac-arm64' => 'aarch64-apple-darwin',
            'linux-x64' => 'x86_64-unknown-linux-gnu',
            'linux-arm64' => 'aarch64-unknown-linux-gnu',
            default => null, // If null, tauri uses host target
        };
    }

    private function copyPhpSidecar(?string $target): void
    {
        $this->newLine();
        intro('Copying PHP sidecar binary...');

        $tauriPath = TauriServiceProvider::tauriPath();
        $sidecarDir = $tauriPath . '/src-tauri/bin';

        if (!is_dir($sidecarDir)) {
            mkdir($sidecarDir, 0755, true);
        }

        if (!$target) {
            $target = Process::run('rustc -vV | grep host | cut -d" " -f2')->output();
            $target = trim($target);
        }

        // Determine source PHP binary based on OS and architecture
        $os = match (true) {
            str_contains($target, 'windows') => 'win',
            str_contains($target, 'apple') => 'mac',
            str_contains($target, 'linux') => 'linux',
            default => 'mac',
        };

        $arch = match (true) {
            str_contains($target, 'aarch64') => 'arm64',
            str_contains($target, 'x86_64') => 'x64',
            default => 'x64',
        };

        $ext = $os === 'win' ? '.exe' : '';
        $sourceBinary = $this->builder->phpBinaryPath() . "{$os}/{$arch}/php{$ext}";
        
        $destBinary = $sidecarDir . "/php-{$target}{$ext}";

        if (file_exists($sourceBinary)) {
            copy($sourceBinary, $destBinary);
            chmod($destBinary, 0755);
        } else {
            $this->error("Warning: Bundled PHP binary not found at {$sourceBinary}. Tauri build might fail or use system PHP.");
        }
    }
}
