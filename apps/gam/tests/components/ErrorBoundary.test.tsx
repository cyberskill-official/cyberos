import { render, screen } from '@testing-library/react';

import { ErrorBoundary } from '#/components/ErrorBoundary';

// Component that throws an error
function ThrowError({ shouldThrow }: { shouldThrow: boolean }) {
    if (shouldThrow) {
        throw new Error('Test error message');
    }

    return <div>Normal content</div>;
}

describe('errorBoundary', () => {
    // Suppress console.error noise from React error boundary
    const originalError = console.error;

    beforeEach(() => {
        console.error = vi.fn();
    });

    afterEach(() => {
        console.error = originalError;
    });

    it('renders children when no error', () => {
        render(
            <ErrorBoundary>
                <ThrowError shouldThrow={false} />
            </ErrorBoundary>,
        );

        expect(screen.getByText('Normal content')).toBeInTheDocument();
    });

    it('renders error fallback when child throws', () => {
        render(
            <ErrorBoundary>
                <ThrowError shouldThrow={true} />
            </ErrorBoundary>,
        );

        expect(screen.getByText('Something went wrong')).toBeInTheDocument();
        expect(screen.getByText('Test error message')).toBeInTheDocument();
    });

    it('shows Try Again button in error state', () => {
        render(
            <ErrorBoundary>
                <ThrowError shouldThrow={true} />
            </ErrorBoundary>,
        );

        expect(screen.getByText('Try Again')).toBeInTheDocument();
    });
});
