<script lang="ts">
  import { ArrowLeft, Plus, Search, User, Trash2, Edit3, Copy, Check } from "lucide-svelte";
  import { ui } from "../lib/stores/ui";
  import { copyToClipboard, truncateAddress } from "../lib/utils/format";
  import Button from "../lib/components/Button.svelte";
  import Input from "../lib/components/Input.svelte";

  interface Contact {
    id: string;
    name: string;
    address: string;
  }

  // Local storage key
  const STORAGE_KEY = "ikki_contacts";

  // Load contacts from local storage
  function loadContacts(): Contact[] {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  // Save contacts to local storage
  function saveContacts(contacts: Contact[]) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(contacts));
  }

  let contacts: Contact[] = loadContacts();
  let searchQuery = "";
  let showAddForm = false;
  let editingId: string | null = null;
  let newName = "";
  let newAddress = "";
  let copiedId: string | null = null;

  function handleBack() {
    if (showAddForm || editingId) {
      showAddForm = false;
      editingId = null;
      newName = "";
      newAddress = "";
    } else {
      ui.navigate("settings");
    }
  }

  function startAdd() {
    showAddForm = true;
    editingId = null;
    newName = "";
    newAddress = "";
  }

  function startEdit(contact: Contact) {
    editingId = contact.id;
    showAddForm = false;
    newName = contact.name;
    newAddress = contact.address;
  }

  function handleSave() {
    const trimmedName = newName.trim();
    const trimmedAddress = newAddress.trim();

    if (!trimmedName) {
      ui.showToast("Please enter a name", "error");
      return;
    }

    if (!trimmedAddress) {
      ui.showToast("Please enter an address", "error");
      return;
    }

    // Basic address validation (should start with u, t, or z for Zcash)
    if (!/^[utzs]/.test(trimmedAddress)) {
      ui.showToast("Invalid Zcash address format", "error");
      return;
    }

    if (editingId) {
      // Update existing contact
      contacts = contacts.map((c) =>
        c.id === editingId
          ? { ...c, name: trimmedName, address: trimmedAddress }
          : c
      );
      ui.showToast("Contact updated", "success");
    } else {
      // Add new contact
      const newContact: Contact = {
        id: Math.random().toString(36).substring(2, 9),
        name: trimmedName,
        address: trimmedAddress,
      };
      contacts = [...contacts, newContact];
      ui.showToast("Contact added", "success");
    }

    saveContacts(contacts);
    showAddForm = false;
    editingId = null;
    newName = "";
    newAddress = "";
  }

  function handleDelete(id: string) {
    contacts = contacts.filter((c) => c.id !== id);
    saveContacts(contacts);
    ui.showToast("Contact deleted", "success");
    if (editingId === id) {
      editingId = null;
      newName = "";
      newAddress = "";
    }
  }

  async function handleCopy(address: string, id: string) {
    const success = await copyToClipboard(address);
    if (success) {
      copiedId = id;
      ui.showToast("Address copied", "success");
      setTimeout(() => (copiedId = null), 2000);
    }
  }

  function handleSelectContact(contact: Contact) {
    // Store selected contact and navigate to send
    sessionStorage.setItem("selected_recipient", JSON.stringify(contact));
    ui.navigate("send");
  }

  function handleSearchInput(e: Event) {
    const target = e.target as HTMLInputElement;
    searchQuery = target.value;
  }

  function handleNameInput(e: Event) {
    const target = e.target as HTMLInputElement;
    newName = target.value;
  }

  function handleAddressInput(e: Event) {
    const target = e.target as HTMLInputElement;
    newAddress = target.value;
  }

  $: filteredContacts = contacts.filter(
    (c) =>
      c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      c.address.toLowerCase().includes(searchQuery.toLowerCase())
  );
</script>

<div class="contacts">
  <header class="contacts-header">
    <button class="back-button" onclick={handleBack}>
      <ArrowLeft size={20} strokeWidth={2} />
    </button>
    <h1>{showAddForm ? "Add Contact" : editingId ? "Edit Contact" : "Contacts"}</h1>
    {#if !showAddForm && !editingId}
      <button class="add-button" onclick={startAdd}>
        <Plus size={20} strokeWidth={2} />
      </button>
    {:else}
      <div class="header-spacer"></div>
    {/if}
  </header>

  <div class="contacts-content">
    {#if showAddForm || editingId}
      <!-- Add/Edit Form -->
      <div class="form-section">
        <Input
          type="text"
          label="Name"
          placeholder="Contact name"
          value={newName}
          oninput={handleNameInput}
        />

        <div class="address-input-container">
          <Input
            type="text"
            label="Address"
            placeholder="Zcash address (u1..., t1..., zs1...)"
            value={newAddress}
            oninput={handleAddressInput}
          />
        </div>

        <div class="form-actions">
          <Button variant="primary" size="lg" fullWidth onclick={handleSave}>
            {editingId ? "Update Contact" : "Add Contact"}
          </Button>
          {#if editingId}
            <Button
              variant="danger"
              size="lg"
              fullWidth
              onclick={() => handleDelete(editingId!)}
            >
              <Trash2 size={16} />
              Delete Contact
            </Button>
          {/if}
        </div>
      </div>
    {:else}
      <!-- Search -->
      {#if contacts.length > 0}
        <div class="search-section">
          <div class="search-box">
            <Search size={16} class="search-icon" />
            <input
              type="text"
              class="search-input"
              placeholder="Search contacts..."
              value={searchQuery}
              oninput={handleSearchInput}
            />
          </div>
        </div>
      {/if}

      <!-- Contact List -->
      <div class="contact-list">
        {#if filteredContacts.length === 0}
          <div class="empty-state">
            {#if contacts.length === 0}
              <User size={48} strokeWidth={1} />
              <h3>No Contacts</h3>
              <p>Add contacts to quickly send ZEC to saved addresses.</p>
              <Button variant="primary" onclick={startAdd}>
                <Plus size={16} />
                Add Contact
              </Button>
            {:else}
              <Search size={32} strokeWidth={1.5} />
              <h3>No Results</h3>
              <p>No contacts match your search.</p>
            {/if}
          </div>
        {:else}
          {#each filteredContacts as contact (contact.id)}
            <div class="contact-item">
              <button class="contact-main" onclick={() => handleSelectContact(contact)}>
                <div class="contact-avatar">
                  <User size={18} strokeWidth={2} />
                </div>
                <div class="contact-info">
                  <span class="contact-name">{contact.name}</span>
                  <span class="contact-address">{truncateAddress(contact.address, 8)}</span>
                </div>
              </button>
              <div class="contact-actions">
                <button
                  class="action-btn"
                  onclick={() => handleCopy(contact.address, contact.id)}
                  title="Copy address"
                >
                  {#if copiedId === contact.id}
                    <Check size={14} strokeWidth={2.5} />
                  {:else}
                    <Copy size={14} strokeWidth={2} />
                  {/if}
                </button>
                <button
                  class="action-btn"
                  onclick={() => startEdit(contact)}
                  title="Edit contact"
                >
                  <Edit3 size={14} strokeWidth={2} />
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .contacts {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  .contacts-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .contacts-header h1 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-wide);
  }

  .back-button,
  .add-button {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: var(--radius-md);
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
  }

  .back-button:hover,
  .add-button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .header-spacer {
    width: 40px;
  }

  .contacts-content {
    flex: 1;
    padding: var(--space-5);
    max-width: var(--max-width);
    margin: 0 auto;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  /* Search */
  .search-section {
    margin-bottom: var(--space-2);
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: border-color var(--duration-fast) var(--ease-out);
  }

  .search-box:focus-within {
    border-color: var(--border-focus);
  }

  .search-box :global(.search-icon) {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: var(--text-sm);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-disabled);
  }

  /* Contact List */
  .contact-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .contact-item {
    display: flex;
    align-items: center;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    position: relative;
  }

  .contact-item::before {
    content: "";
    position: absolute;
    inset: 0;
    background: var(--gradient-card);
    pointer-events: none;
    border-radius: inherit;
  }

  .contact-main {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    position: relative;
    transition: background var(--duration-fast) var(--ease-out);
  }

  .contact-main:hover {
    background: var(--bg-hover);
  }

  .contact-avatar {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-elevated);
    border-radius: var(--radius-full);
    color: var(--text-tertiary);
    border: 1px solid var(--border);
  }

  .contact-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .contact-name {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    letter-spacing: var(--tracking-normal);
  }

  .contact-address {
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    letter-spacing: var(--tracking-wide);
  }

  .contact-actions {
    display: flex;
    gap: var(--space-1);
    padding-right: var(--space-3);
    position: relative;
  }

  .action-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
  }

  .action-btn:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }

  /* Empty State */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-8);
    color: var(--text-tertiary);
    gap: var(--space-4);
  }

  .empty-state h3 {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .empty-state p {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    max-width: 260px;
    line-height: var(--leading-relaxed);
    margin: 0;
  }

  /* Form */
  .form-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .address-input-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .form-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-top: var(--space-4);
  }
</style>
