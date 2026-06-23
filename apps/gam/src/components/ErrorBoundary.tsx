import * as React from 'react';

interface I_ErrorBoundaryProps {
    children: React.ReactNode;
    onRetry?: () => void;
}

interface I_ErrorBoundaryState {
    hasError: boolean;
    error: Error | null;
}

export class ErrorBoundary extends React.Component<I_ErrorBoundaryProps, I_ErrorBoundaryState> {
    constructor(props: I_ErrorBoundaryProps) {
        super(props);
        this.state = { hasError: false, error: null };
    }

    static getDerivedStateFromError(error: Error) {
        return { hasError: true, error };
    }

    handleReset = () => {
        this.setState({ hasError: false, error: null });
        this.props.onRetry?.();
    };

    render() {
        if (this.state.hasError) {
            return (
                <div className="error-boundary-container">
                    <div className="error-boundary-card">
                        <h2 className="text-xl font-bold mb-3">Something went wrong</h2>
                        <div className="error-boundary-detail">
                            <p className="font-mono text-sm m-0 error-boundary-message">{this.state.error?.message}</p>
                        </div>
                        <button
                            className="px-5 py-1.5 font-bold bg-transparent border rounded cursor-pointer transition-all hover:-translate-y-0.5 error-boundary-btn"
                            onClick={this.handleReset}
                        >
                            Try Again
                        </button>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}
