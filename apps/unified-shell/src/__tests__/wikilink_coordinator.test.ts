import { describe, it, expect, beforeEach } from 'vitest';
import {
  WikiLinkCoordinator,
  type UnlinkedMention,
} from '../state/WikiLinkCoordinator';

describe('WikiLinkCoordinator - Bidirectional [[WikiLinks]] & Backlinks (Sub-Phase 50.1.2)', () => {
  let coordinator: WikiLinkCoordinator;

  beforeEach(() => {
    coordinator = new WikiLinkCoordinator();
  });

  it('1. Initializes cleanly with default state', () => {
    const state = coordinator.getState();
    expect(state.activeDocId).toBeNull();
    expect(state.activeDocTitle).toBeNull();
    expect(state.linkedReferences).toEqual([]);
    expect(state.forwardLinks).toEqual([]);
    expect(state.unlinkedMentions).toEqual([]);
    expect(state.suggestions).toEqual([]);
    expect(state.stats.total_documents).toBe(0);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('2. Indexes a document and updates index stats', async () => {
    await coordinator.indexDocument(
      'doc-notes',
      'Meeting Notes',
      'Discussed [[Project Roadmap]] and [[Design System|UI Tokens]].'
    );

    const state = coordinator.getState();
    expect(state.stats.total_documents).toBeGreaterThanOrEqual(1);
    expect(state.stats.total_wikilinks).toBeGreaterThanOrEqual(2);
    expect(state.error).toBeNull();
  });

  it('3. Sets active document and retrieves linked references', async () => {
    await coordinator.indexDocument(
      'doc-t3-notes-1',
      'Meeting Notes T3',
      'Key deliverables depend on [[Alpha Roadmap]].'
    );
    await coordinator.indexDocument(
      'doc-t3-roadmap',
      'Alpha Roadmap',
      'Master product milestones for Q4.'
    );

    await coordinator.setActiveDocument('doc-t3-roadmap', 'Alpha Roadmap');
    const state = coordinator.getState();

    expect(state.activeDocId).toBe('doc-t3-roadmap');
    expect(state.linkedReferences.length).toBeGreaterThanOrEqual(1);
    expect(state.linkedReferences[0].source_doc_id).toBe('doc-t3-notes-1');
    expect(state.linkedReferences[0].snippet).toContain('Alpha Roadmap');
  });

  it('4. Retrieves forward links for the active document', async () => {
    await coordinator.indexDocument(
      'doc-spec',
      'Technical Spec',
      'References [[Architecture Guild#Decisions|Decisions]] and [[Security Whitepaper]].'
    );

    await coordinator.setActiveDocument('doc-spec', 'Technical Spec');
    const state = coordinator.getState();

    expect(state.forwardLinks.length).toBe(2);
    expect(state.forwardLinks[0].target_doc).toBe('Architecture Guild');
    expect(state.forwardLinks[0].target_anchor).toBe('Decisions');
    expect(state.forwardLinks[0].display_text).toBe('Decisions');
    expect(state.forwardLinks[1].target_doc).toBe('Security Whitepaper');
  });

  it('5. Queries autocomplete suggestions with fuzzy scoring', async () => {
    await coordinator.indexDocument(
      'doc-roadmap',
      'Project Roadmap',
      'Product timelines.'
    );
    await coordinator.indexDocument(
      'doc-sprint',
      'Sprint Planning',
      'Task assignments.'
    );

    const suggestions = await coordinator.querySuggestions('Roadmap', 5);
    expect(suggestions.length).toBeGreaterThanOrEqual(1);
    expect(suggestions.some((s) => s.title === 'Project Roadmap')).toBe(true);
    expect(suggestions[0].score).toBeGreaterThan(0);
  });

  it('6. Formats wikilinks with anchors and aliases', () => {
    const simple = coordinator.formatWikiLink('Roadmap');
    expect(simple).toBe('[[Roadmap]]');

    const anchored = coordinator.formatWikiLink('Roadmap', 'Phase 1');
    expect(anchored).toBe('[[Roadmap#Phase 1]]');

    const aliased = coordinator.formatWikiLink('Roadmap', null, 'Target Dates');
    expect(aliased).toBe('[[Roadmap|Target Dates]]');

    const full = coordinator.formatWikiLink('Roadmap', 'Phase 1', 'Q4 Delivery');
    expect(full).toBe('[[Roadmap#Phase 1|Q4 Delivery]]');
  });

  it('7. Discovers unlinked mentions in workspace documents', async () => {
    await coordinator.indexDocument(
      'doc-guidelines',
      'Design System',
      'Visual language and component library.'
    );
    await coordinator.indexDocument(
      'doc-blog',
      'Blog Post Draft',
      'We redesigned our entire user interface following our new Design System principles.'
    );

    await coordinator.setActiveDocument('doc-guidelines', 'Design System');
    const state = coordinator.getState();

    expect(state.unlinkedMentions.length).toBeGreaterThanOrEqual(1);
    expect(state.unlinkedMentions[0].source_doc_id).toBe('doc-blog');
    expect(state.unlinkedMentions[0].matched_text).toBe('Design System');
  });

  it('8. Converts unlinked mention into an explicit [[WikiLink]]', () => {
    const sourceContent = 'Refer to Design System for typography standards.';
    const mention: UnlinkedMention = {
      source_doc_id: 'doc-blog',
      source_doc_title: 'Blog Post Draft',
      matched_text: 'Design System',
      snippet: 'Refer to Design System for typography...',
      start_offset: 9,
      end_offset: 22,
    };

    const converted = coordinator.convertMentionToWikiLink(sourceContent, mention);
    expect(converted).toBe('Refer to [[Design System]] for typography standards.');
  });

  it('9. Removes document from index and cleans references', async () => {
    await coordinator.indexDocument(
      'doc-temp',
      'Temporary Notes',
      'Points to [[Shared Knowledge]].'
    );
    await coordinator.indexDocument(
      'doc-shared',
      'Shared Knowledge',
      'Common repository.'
    );

    await coordinator.setActiveDocument('doc-shared', 'Shared Knowledge');
    expect(coordinator.getState().linkedReferences.length).toBeGreaterThanOrEqual(1);

    await coordinator.removeDocument('doc-temp');
    await coordinator.refreshActiveDocumentLinks();
    expect(coordinator.getState().linkedReferences.length).toBe(0);
  });

  it('10. Clears state and notifies reactive subscribers', () => {
    let callCount = 0;
    const unsub = coordinator.subscribe(() => {
      callCount++;
    });

    coordinator.clear();
    const state = coordinator.getState();
    expect(state.activeDocId).toBeNull();
    expect(state.linkedReferences).toEqual([]);
    expect(state.stats.total_documents).toBe(0);
    expect(callCount).toBeGreaterThanOrEqual(1);

    unsub();
  });
});
