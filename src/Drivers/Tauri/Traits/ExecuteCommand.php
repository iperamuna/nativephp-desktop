<?php

namespace Native\Desktop\Drivers\Tauri\Traits;

use Illuminate\Support\Facades\Process;
use Native\Desktop\Builder\Builder;
use Native\Desktop\Drivers\Tauri\TauriServiceProvider;

use function Laravel\Prompts\error;

trait ExecuteCommand
{
    protected function executeCommand(
        string $command,
        bool $skip_queue = false,
        string $type = 'install',
        bool $no_focus = false,
        bool $withoutInteraction = false
    ): void {

        $builder = resolve(Builder::class);

        $envs = [
            'install' => [
                'NATIVEPHP_PHP_BINARY_VERSION' => PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION,
                'NATIVEPHP_PHP_BINARY_PATH' => $builder->phpBinaryPath(),
            ],
            'serve' => [
                'APP_PATH' => base_path(),
                'NATIVEPHP_PHP_BINARY_VERSION' => PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION,
                'NATIVEPHP_PHP_BINARY_PATH' => $builder->phpBinaryPath(),
                'NATIVE_PHP_SKIP_QUEUE' => $skip_queue,
                'NATIVEPHP_BUILDING' => false,
                'NATIVEPHP_TAURI_PATH' => TauriServiceProvider::tauriPath(),
                'NATIVEPHP_BUILD_PATH' => TauriServiceProvider::buildPath(),
                'NATIVEPHP_NO_FOCUS' => $no_focus,
            ],
        ];

        $result = Process::path(TauriServiceProvider::tauriPath())
            ->env($envs[$type] ?? [])
            ->forever()
            ->tty(! $withoutInteraction && PHP_OS_FAMILY != 'Windows')
            ->run($command, function (string $type, string $output) {
                if ($this->getOutput()->isVerbose()) {
                    echo $output;
                }
            });

        // Don't throw. PHP Exception won't give any valuable info.
        // Error lines already echoed in the process output.
        if ($result->failed()) {
            echo PHP_EOL;
            error("Command failed: '{$command}' (exit code {$result->exitCode()})");
            exit($result->exitCode());
        }
    }

    protected function getCommandArrays(string $type = 'install'): array
    {
        $commands = [
            'install' => [
                'npm' => 'npm install',
                'yarn' => 'yarn',
                'pnpm' => 'pnpm install',
            ],
            'dev' => [
                'npm' => 'npm run tauri dev',
                'yarn' => 'yarn tauri dev',
                'pnpm' => 'pnpm tauri dev',
            ],
            'build' => [
                'npm' => 'npm run tauri build',
                'yarn' => 'yarn tauri build',
                'pnpm' => 'pnpm tauri build',
            ],
        ];

        return $commands[$type];
    }
}
