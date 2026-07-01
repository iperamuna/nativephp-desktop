<?php

namespace Native\Desktop\Drivers\Tauri\Commands;

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

        $this->runDeveloper(
            installer: $this->option('installer'),
            skip_queue: $this->option('no-queue'),
            no_focus: $this->option('no-focus'),
            withoutInteraction: $this->option('no-interaction')
        );
    }
}
