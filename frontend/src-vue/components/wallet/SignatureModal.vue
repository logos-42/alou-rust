<template>
  <Transition name="modal">
    <div v-if="show" class="modal-overlay" @click="$emit('cancel')">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>{{ t('signatureRequest') }}</h3>
          <button @click="$emit('cancel')" class="close-btn">✕</button>
        </div>
        <div class="modal-body">
          <div class="signature-info">
            <div class="info-item">
              <span class="label">{{ t('from') }}:</span>
              <span class="value">{{ formatAddress(request.from) }}</span>
            </div>
            <div class="info-item">
              <span class="label">{{ t('to') }}:</span>
              <span class="value">{{ formatAddress(request.to) }}</span>
            </div>
            <div class="info-item">
              <span class="label">{{ t('amount') }}:</span>
              <span class="value highlight">{{ request.value }} {{ request.token }}</span>
            </div>
            <div class="info-item">
              <span class="label">{{ t('gasFee') }}:</span>
              <span class="value">{{ request.gasFee }} ETH</span>
            </div>
          </div>
          <div class="warning-box">
            <span class="warning-icon">⚠️</span>
            <span>{{ t('signatureWarning') }}</span>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="$emit('cancel')" class="btn-secondary">
            {{ t('cancel') }}
          </button>
          <button @click="$emit('confirm')" class="btn-primary" :disabled="isSigning">
            <span v-if="isSigning">{{ t('signing') }}...</span>
            <span v-else>{{ t('confirm') }}</span>
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n'

interface SignatureRequest {
  from: string
  to: string
  value: string
  token: string
  gasFee: string
  data?: string
}

interface Props {
  show: boolean
  request: SignatureRequest
  isSigning: boolean
}

defineProps<Props>()
defineEmits(['confirm', 'cancel'])

const { t } = useI18n()

function formatAddress(address: string): string {
  if (!address || address.length <= 10) return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

.modal-content {
  background: var(--background);
  border-radius: 1.5rem;
  max-width: 500px;
  width: 100%;
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem 2rem;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 1.5rem;
  color: var(--text-secondary);
  cursor: pointer;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.5rem;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: var(--secondary-color);
  color: var(--text-primary);
}

.modal-body {
  padding: 2rem;
}

.signature-info {
  background: var(--surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border-color);
}

.info-item:last-child {
  border-bottom: none;
}

.label {
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-weight: 500;
}

.value {
  font-size: 0.875rem;
  color: var(--text-primary);
  font-weight: 600;
  font-family: monospace;
}

.value.highlight {
  color: var(--primary-color);
  font-size: 1.125rem;
}

.warning-box {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid var(--warning-color);
  border-radius: 0.75rem;
  font-size: 0.875rem;
  color: var(--text-primary);
}

.warning-icon {
  font-size: 1.5rem;
  flex-shrink: 0;
}

.modal-footer {
  display: flex;
  gap: 1rem;
  padding: 1.5rem 2rem;
  border-top: 1px solid var(--border-color);
}

.btn-secondary,
.btn-primary {
  flex: 1;
  padding: 1rem;
  border-radius: 0.75rem;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-secondary {
  background: var(--surface);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
}

.btn-secondary:hover {
  background: var(--secondary-color);
}

.btn-primary {
  background: var(--primary-color);
  border: none;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-hover);
  transform: translateY(-2px);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition: transform 0.3s ease;
}

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: scale(0.9);
}
</style>
