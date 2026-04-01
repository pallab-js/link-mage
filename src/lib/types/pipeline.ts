/**
 * Transform Chaining Pipelines
 * 
 * Allows users to define and execute multi-step transform sequences.
 * Inspired by Maltego's "Machines" feature and workflow automation tools.
 */

import { writable, derived, get } from 'svelte/store';
import type { Writable } from 'svelte/store';

/**
 * Pipeline step configuration
 */
export interface PipelineStep {
  /** Unique step identifier */
  id: string;
  /** Name of the transform to execute */
  transformName: string;
  /** Expected input node type (empty means any type) */
  inputType: string;
  /** Description of what this step does */
  description: string;
  /** Timeout in seconds (0 means use default) */
  timeout: number;
  /** Whether to continue on error */
  continueOnError: boolean;
  /** Maximum number of results to process (0 means unlimited) */
  maxResults: number;
}

/**
 * Pipeline execution status
 */
export type PipelineStatus = 
  | 'idle'
  | 'running'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled';

/**
 * Step execution result
 */
export interface StepResult {
  stepId: string;
  transformName: string;
  status: 'pending' | 'running' | 'success' | 'failed' | 'skipped';
  nodesAdded: number;
  edgesAdded: number;
  error?: string;
  durationMs: number;
  startedAt: string;
  completedAt?: string;
}

/**
 * Pipeline execution log
 */
export interface PipelineExecution {
  id: string;
  pipelineId: string;
  pipelineName: string;
  seedNodeId: string;
  seedNodeType: string;
  status: PipelineStatus;
  steps: StepResult[];
  totalNodesAdded: number;
  totalEdgesAdded: number;
  startedAt: string;
  completedAt?: string;
  error?: string;
}

/**
 * Pipeline definition
 */
export interface Pipeline {
  /** Unique identifier */
  id: string;
  /** Pipeline name */
  name: string;
  /** Pipeline description */
  description: string;
  /** Pipeline steps in execution order */
  steps: PipelineStep[];
  /** Creation timestamp (ISO 8601) */
  createdAt: string;
  /** Last modified timestamp (ISO 8601) */
  updatedAt: string;
  /** Number of times pipeline has been executed */
  executionCount: number;
  /** Whether pipeline is enabled */
  enabled: boolean;
  /** Tags for categorization */
  tags: string[];
}

/**
 * Pre-built pipeline templates
 */
export const PIPELINE_TEMPLATES: Array<{
  id: string;
  name: string;
  description: string;
  steps: Omit<PipelineStep, 'id'>[];
  tags: string[];
}> = [
  {
    id: 'domain-enrichment',
    name: 'Domain Enrichment',
    description: 'Comprehensive domain intelligence gathering',
    steps: [
      { transformName: 'DNS Lookup', inputType: 'Domain', description: 'Resolve to IP addresses', timeout: 30, continueOnError: false, maxResults: 0 },
      { transformName: 'GeoIP Lookup', inputType: 'IP', description: 'Geolocate IP addresses', timeout: 30, continueOnError: true, maxResults: 0 },
      { transformName: 'WHOIS Lookup', inputType: 'Domain', description: 'Get domain registration info', timeout: 30, continueOnError: true, maxResults: 0 },
      { transformName: 'Reverse DNS', inputType: 'IP', description: 'Find domains on IP', timeout: 30, continueOnError: true, maxResults: 0 },
    ],
    tags: ['dns', 'domain', 'enrichment'],
  },
  {
    id: 'email-investigation',
    name: 'Email Investigation',
    description: 'Investigate email addresses and find associated data',
    steps: [
      { transformName: 'Domain Search', inputType: 'Email', description: 'Extract domain from email', timeout: 10, continueOnError: false, maxResults: 0 },
      { transformName: 'DNS Lookup', inputType: 'Domain', description: 'Resolve domain', timeout: 30, continueOnError: true, maxResults: 0 },
      { transformName: 'WHOIS Lookup', inputType: 'Domain', description: 'Get domain owner', timeout: 30, continueOnError: true, maxResults: 0 },
    ],
    tags: ['email', 'investigation'],
  },
  {
    id: 'ip-reconnaissance',
    name: 'IP Reconnaissance',
    description: 'Gather intelligence on IP addresses',
    steps: [
      { transformName: 'GeoIP Lookup', inputType: 'IP', description: 'Geolocate IP', timeout: 30, continueOnError: false, maxResults: 0 },
      { transformName: 'Reverse DNS', inputType: 'IP', description: 'Find hostnames', timeout: 30, continueOnError: true, maxResults: 0 },
      { transformName: 'VirusTotal IP', inputType: 'IP', description: 'Check reputation', timeout: 30, continueOnError: true, maxResults: 0 },
    ],
    tags: ['ip', 'recon', 'security'],
  },
  {
    id: 'malware-analysis',
    name: 'Malware Analysis',
    description: 'Analyze file hashes across security vendors',
    steps: [
      { transformName: 'VirusTotal Hash', inputType: 'Hash', description: 'Check hash reputation', timeout: 60, continueOnError: false, maxResults: 0 },
    ],
    tags: ['malware', 'hash', 'security'],
  },
];

// ── Pipeline Store ────────────────────────────────────────────────────────────

function createPipelineStore() {
  const { subscribe, set, update }: Writable<Pipeline[]> = writable([]);
  const executionsStore: Writable<PipelineExecution[]> = writable([]);
  const currentExecutionStore: Writable<PipelineExecution | null> = writable(null);

  return {
    subscribe,

    /** Get executions store for subscription */
    get executions() {
      return executionsStore;
    },

    /** Get current execution store */
    get currentExecution() {
      return currentExecutionStore;
    },

    /** Initialize with existing pipelines */
    initialize(pipelines: Pipeline[]) {
      set(pipelines);
    },

    /** Create a new pipeline */
    create(pipeline: Omit<Pipeline, 'id' | 'createdAt' | 'updatedAt' | 'executionCount'>): Pipeline {
      const now = new Date().toISOString();
      const newPipeline: Pipeline = {
        ...pipeline,
        id: generateId(),
        createdAt: now,
        updatedAt: now,
        executionCount: 0,
      };
      update(pipes => [...pipes, newPipeline]);
      return newPipeline;
    },

    /** Update an existing pipeline */
    update(id: string, updates: Partial<Pipeline>): boolean {
      let updated = false;
      update(pipes => 
        pipes.map(p => {
          if (p.id === id) {
            updated = true;
            return { ...p, ...updates, updatedAt: new Date().toISOString() };
          }
          return p;
        })
      );
      return updated;
    },

    /** Delete a pipeline */
    remove(id: string): boolean {
      let removed = false;
      update(pipes => {
        const filtered = pipes.filter(p => {
          if (p.id === id) {
            removed = true;
            return false;
          }
          return true;
        });
        return filtered;
      });
      return removed;
    },

    /** Get a pipeline by ID */
    getById(id: string): Pipeline | undefined {
      const pipes = get({ subscribe });
      return pipes.find(p => p.id === id);
    },

    /** Get all enabled pipelines */
    getEnabled(): Pipeline[] {
      const pipes = get({ subscribe });
      return pipes.filter(p => p.enabled);
    },

    /** Get pipelines by tag */
    getByTag(tag: string): Pipeline[] {
      const pipes = get({ subscribe });
      return pipes.filter(p => p.tags.includes(tag));
    },

    /** Increment execution count */
    incrementExecution(id: string) {
      update(pipes => 
        pipes.map(p => 
          p.id === id ? { ...p, executionCount: p.executionCount + 1, updatedAt: new Date().toISOString() } : p
        )
      );
    },

    /** Start a pipeline execution */
    startExecution(pipeline: Pipeline, seedNodeId: string, seedNodeType: string): PipelineExecution {
      const execution: PipelineExecution = {
        id: generateId(),
        pipelineId: pipeline.id,
        pipelineName: pipeline.name,
        seedNodeId,
        seedNodeType,
        status: 'running',
        steps: pipeline.steps.map(s => ({
          stepId: s.id,
          transformName: s.transformName,
          status: 'pending',
          nodesAdded: 0,
          edgesAdded: 0,
          durationMs: 0,
          startedAt: new Date().toISOString(),
        })),
        totalNodesAdded: 0,
        totalEdgesAdded: 0,
        startedAt: new Date().toISOString(),
      };

      executionsStore.update(ex => [...ex, execution]);
      currentExecutionStore.set(execution);
      return execution;
    },

    /** Update step status */
    updateStep(executionId: string, stepId: string, updates: Partial<StepResult>) {
      executionsStore.update(ex => 
        ex.map(e => {
          if (e.id === executionId) {
            const updatedSteps = e.steps.map(s => 
              s.stepId === stepId ? { ...s, ...updates } : s
            );
            return { ...e, steps: updatedSteps };
          }
          return e;
        })
      );

      // Also update current execution
      const current = get(currentExecutionStore);
      if (current && current.id === executionId) {
        const updatedSteps = current.steps.map(s => 
          s.stepId === stepId ? { ...s, ...updates } : s
        );
        currentExecutionStore.set({ ...current, steps: updatedSteps });
      }
    },

    /** Update execution status */
    updateExecution(executionId: string, updates: Partial<PipelineExecution>) {
      executionsStore.update(ex => 
        ex.map(e => e.id === executionId ? { ...e, ...updates } : e)
      );

      const current = get(currentExecutionStore);
      if (current && current.id === executionId) {
        currentExecutionStore.set({ ...current, ...updates });
      }
    },

    /** Clear current execution */
    clearCurrentExecution() {
      currentExecutionStore.set(null);
    },

    /** Get execution history */
    getExecutionHistory(limit = 50): PipelineExecution[] {
      const executions = get(executionsStore);
      return executions
        .sort((a, b) => new Date(b.startedAt).getTime() - new Date(a.startedAt).getTime())
        .slice(0, limit);
    },

    /** Clear all data */
    clear() {
      set([]);
      executionsStore.set([]);
      currentExecutionStore.set(null);
    },

    /** Export for persistence */
    export() {
      return {
        pipelines: get({ subscribe }),
        executions: get(executionsStore),
      };
    },

    /** Import from persistence */
    import(data: { pipelines: Pipeline[]; executions: PipelineExecution[] }) {
      set(data.pipelines);
      executionsStore.set(data.executions);
    },
  };
}

export const pipelineStore = createPipelineStore();

// ── Utilities ─────────────────────────────────────────────────────────────────

function generateId(): string {
  return `pipe_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

function generateStepId(): string {
  return `step_${Date.now()}_${Math.random().toString(36).substr(2, 6)}`;
}

/**
 * Create a new pipeline step with defaults
 */
export function createPipelineStep(
  transformName: string,
  inputType: string = '',
  description: string = '',
): PipelineStep {
  return {
    id: generateStepId(),
    transformName,
    inputType,
    description,
    timeout: 30,
    continueOnError: true,
    maxResults: 0,
  };
}

/**
 * Create a pipeline from a template
 */
export function createPipelineFromTemplate(templateId: string, name?: string): Pipeline | null {
  const template = PIPELINE_TEMPLATES.find(t => t.id === templateId);
  if (!template) return null;

  const now = new Date().toISOString();
  const steps: PipelineStep[] = template.steps.map(s => ({
    ...s,
    id: generateStepId(),
  }));

  return {
    id: generateId(),
    name: name || template.name,
    description: template.description,
    steps,
    createdAt: now,
    updatedAt: now,
    executionCount: 0,
    enabled: true,
    tags: template.tags,
  };
}

/**
 * Validate pipeline configuration
 */
export function validatePipeline(pipeline: Partial<Pipeline>): string[] {
  const errors: string[] = [];

  if (!pipeline.name || pipeline.name.trim().length === 0) {
    errors.push('Pipeline name is required');
  }

  if (!pipeline.steps || pipeline.steps.length === 0) {
    errors.push('Pipeline must have at least one step');
  } else {
    pipeline.steps.forEach((step, index) => {
      if (!step.transformName || step.transformName.trim().length === 0) {
        errors.push(`Step ${index + 1}: Transform name is required`);
      }
      if (step.timeout < 0 || step.timeout > 300) {
        errors.push(`Step ${index + 1}: Timeout must be between 0 and 300 seconds`);
      }
      if (step.maxResults < 0) {
        errors.push(`Step ${index + 1}: Max results cannot be negative`);
      }
    });
  }

  return errors;
}

/**
 * Estimate pipeline execution time based on steps
 */
export function estimateExecutionTime(steps: PipelineStep[]): number {
  return steps.reduce((total, step) => {
    return total + (step.timeout > 0 ? step.timeout : 30);
  }, 0);
}
